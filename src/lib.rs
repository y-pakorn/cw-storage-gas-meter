use std::cell::RefCell;

use cosmwasm_std::{MemoryStorage, Storage};

/// Same as `cosmwasm_std::MemoryStorage` but has additional gas logging
#[derive(Default, Debug)]
pub struct MemoryStorageWithGas {
    storage: MemoryStorage,
    pub gas_used: RefCell<StorageGasUsed>,
    pub gas_config: StorageGasConfig,
}

impl MemoryStorageWithGas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_gas_config(gas_config: StorageGasConfig) -> Self {
        Self {
            gas_config,
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn total_gas_used(&self) -> u64 {
        self.gas_used.borrow().total
    }

    #[inline(always)]
    pub fn last_gas_used(&self) -> u64 {
        self.gas_used.borrow().last
    }

    pub fn reset_gas(&self) {
        self.gas_used.borrow_mut().total = 0;
    }

    pub fn log_gas(&self) {
        println!("{:#?}", self.gas_used);
    }
}

/// Helper struct to storage total gas used and storage interaction count
#[derive(Default, Debug, PartialEq)]
pub struct StorageGasUsed {
    total: u64,
    last: u64,
    read_cnt: u64,
    write_cnt: u64,
    delete_cnt: u64,
    iter_next_cnt: u64,
}

/// Constant gas config to store gas info based on sdk's KV store pattern
#[derive(Debug)]
pub struct StorageGasConfig {
    pub has_cost: u64,
    pub delete_cost: u64,
    pub read_cost_flat: u64,
    pub read_cost_per_byte: u64,
    pub write_cost_flat: u64,
    pub write_cost_per_byte: u64,
    pub iter_next_cost_flat: u64,
}

impl Default for StorageGasConfig {
    fn default() -> Self {
        Self {
            has_cost: 1000,
            delete_cost: 1000,
            read_cost_flat: 1000,
            read_cost_per_byte: 3,
            write_cost_flat: 2000,
            write_cost_per_byte: 30,
            iter_next_cost_flat: 30,
        }
    }
}

impl Storage for MemoryStorageWithGas {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.read_cost_flat
                + key.len() as u64 * self.gas_config.read_cost_per_byte;
            gas.total += gas.last;
            gas.read_cnt += 1;
        }

        self.storage.get(key)
    }

    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: cosmwasm_std::Order,
    ) -> Box<dyn Iterator<Item = cosmwasm_std::Record> + 'a> {
        Box::new(self.storage.range(start, end, order).map(|e| {
            {
                let mut gas = self.gas_used.borrow_mut();
                gas.last = self.gas_config.iter_next_cost_flat;
                gas.total += gas.last;
                gas.iter_next_cnt += 1;
            }
            e
        }))
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.write_cost_flat
                + key.len() as u64 * self.gas_config.write_cost_per_byte;
            gas.total += gas.last;
            gas.write_cnt += 1;
        }

        self.storage.set(key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.delete_cost;
            gas.total += gas.last;
            gas.delete_cnt += 1;
        }

        self.storage.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Order, StdResult};
    use cw_storage_plus::Map;
    use std::{error::Error, mem::drop};

    use crate::{MemoryStorageWithGas, StorageGasUsed};

    #[test]
    fn default_gas() -> Result<(), Box<dyn Error>> {
        let storage = MemoryStorageWithGas::default();

        assert_eq!(storage.gas_used.take(), StorageGasUsed::default());

        Ok(())
    }

    #[test]
    fn consume_gas() -> Result<(), Box<dyn Error>> {
        let mut storage = MemoryStorageWithGas::default();
        let map = Map::<u64, Vec<u8>>::new("0");

        // write
        let data = b"hello";
        map.save(&mut storage, 0, &data.to_vec())?;

        let gas = storage.gas_used.borrow();
        assert_eq!(gas.last, 2330);
        assert_eq!(gas.write_cnt, 1);
        drop(gas);

        // read
        let loaded_data = map.load(&storage, 0)?;

        let gas = storage.gas_used.borrow();
        assert_eq!(loaded_data, data);
        assert_eq!(gas.last, 1033);
        assert_eq!(gas.read_cnt, 1);
        drop(gas);

        // iter next
        map.range(&storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        let gas = storage.gas_used.borrow();
        assert_eq!(gas.last, 30);
        assert_eq!(gas.iter_next_cnt, 1);
        drop(gas);

        // delete
        map.remove(&mut storage, 0);

        let gas = storage.gas_used.borrow();
        assert_eq!(gas.last, 1000);
        assert_eq!(gas.delete_cnt, 1);
        drop(gas);

        Ok(())
    }
}
