use cosmwasm_std::{Order, Record, Storage};

use crate::{MemoryStorageWithGas, StorageGasConfig};

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

impl Storage for MemoryStorageWithGas {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let value = self.storage.borrow().get(key);

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
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'a> {
        Box::new(
            self.storage
                .borrow()
                .range(start, end, order)
                .collect::<Vec<_>>()
                .into_iter()
                .map(|e| {
                    {
                        let mut gas = self.gas_used.borrow_mut();
                        gas.last = self.gas_config.iter_next_cost_flat
                            + self.gas_config.read_cost_flat
                            + (e.0.len() + e.1.len()) as u64 * self.gas_config.read_cost_per_byte;
                        gas.total += gas.last;
                        gas.iter_next_cnt += 1;
                    }
                    e
                }),
        )
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.write_cost_flat
                + (key.len() + value.len()) as u64 * self.gas_config.write_cost_per_byte;
            gas.total += gas.last;
            gas.write_cnt += 1;
        }

        self.storage.borrow_mut().set(key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.delete_cost;
            gas.total += gas.last;
            gas.delete_cnt += 1;
        }

        self.storage.borrow_mut().remove(key)
    }
}

impl Storage for &'_ MemoryStorageWithGas {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        MemoryStorageWithGas::get(self, key)
    }

    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record> + 'a> {
        MemoryStorageWithGas::range(self, start, end, order)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.write_cost_flat
                + (key.len() + value.len()) as u64 * self.gas_config.write_cost_per_byte;
            gas.total += gas.last;
            gas.write_cnt += 1;
        }

        self.storage.borrow_mut().set(key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        {
            let mut gas = self.gas_used.borrow_mut();
            gas.last = self.gas_config.delete_cost;
            gas.total += gas.last;
            gas.delete_cnt += 1;
        }

        self.storage.borrow_mut().remove(key)
    }
}
