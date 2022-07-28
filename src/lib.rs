use std::cell::RefCell;

use cosmwasm_std::{MemoryStorage, Storage};

/// A simple storage struct that behave same as [MemoryStorage] but has an additional gas logging.
///
/// More info: <https://github.com/cosmos/cosmos-sdk/blob/main/store/gaskv/store.go>
#[derive(Default, Debug)]
pub struct MemoryStorageWithGas {
    storage: MemoryStorage,
    pub gas_used: RefCell<StorageGasUsed>,
    pub gas_config: StorageGasConfig,
}

impl MemoryStorageWithGas {
    /// Create a new storage instance with default gas config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new storage instance with custom `gas_config` gas config.
    pub fn new_with_gas_config(gas_config: StorageGasConfig) -> Self {
        Self {
            gas_config,
            ..Default::default()
        }
    }

    /// Get total gas usage from current storage instance.
    #[inline(always)]
    pub fn total_gas_used(&self) -> u64 {
        self.gas_used.borrow().total
    }

    /// Get gas usage from latest storage operation.
    #[inline(always)]
    pub fn last_gas_used(&self) -> u64 {
        self.gas_used.borrow().last
    }

    /// Reset current total gas to `0`.
    pub fn reset_gas(&self) {
        self.gas_used.borrow_mut().total = 0;
    }

    /// Log current gas usage into [std::io::stdout].
    pub fn log_gas(&self) {
        println!("{:#?}", self.gas_used);
    }
}

/// Helper struct to store total gas used and interaction count.
///
/// Amount of gas stored in [Self::last] for last gas used and [Self::total] for total gas used.
#[derive(Default, Debug, PartialEq)]
pub struct StorageGasUsed {
    pub total: u64,
    pub last: u64,
    pub read_cnt: u64,
    pub write_cnt: u64,
    pub delete_cnt: u64,
    pub iter_next_cnt: u64,
}

/// Constant gas config struct to store gas info based on sdk's KV store pattern.
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
        let value = self.storage.get(key);

        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.read_cost_flat
                + (key.len() + value.as_ref().unwrap_or(&Vec::new()).len()) as u64
                    * self.gas_config.read_cost_per_byte;
            gas.total += gas.last;
            gas.read_cnt += 1;
        }

        value
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
                gas.last = self.gas_config.iter_next_cost_flat
                    + self.gas_config.read_cost_flat
                    + (e.0.len() + e.1.len()) as u64 * self.gas_config.read_cost_per_byte;
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
                + (key.len() + value.len()) as u64 * self.gas_config.write_cost_per_byte;
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
        assert_eq!(gas.last, 2960);
        assert_eq!(gas.write_cnt, 1);
        drop(gas);

        // read
        let loaded_data = map.load(&storage, 0)?;

        let gas = storage.gas_used.borrow();
        assert_eq!(loaded_data, data);
        assert_eq!(gas.last, 1096);
        assert_eq!(gas.read_cnt, 1);
        drop(gas);

        // iter next
        map.range(&storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        let gas = storage.gas_used.borrow();
        assert_eq!(gas.last, 1126);
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
