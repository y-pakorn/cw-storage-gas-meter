use cosmwasm_std::MemoryStorage;
use std::cell::RefCell;

pub mod impls;

/// A simple storage struct that behave same as [MemoryStorage] but has an additional gas logging.
///
/// More info: <https://github.com/cosmos/cosmos-sdk/blob/main/store/gaskv/store.go>
#[derive(Default, Debug)]
pub struct MemoryStorageWithGas {
    storage: RefCell<MemoryStorage>,
    pub gas_used: RefCell<StorageGasUsed>,
    pub gas_config: StorageGasConfig,
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Coin, Order, StdResult};
    use cw_multi_test::AppBuilder;
    use cw_storage_plus::Map;
    use std::{error::Error, mem::drop};

    use crate::{MemoryStorageWithGas, StorageGasUsed};

    #[test]
    fn default_gas() {
        let storage = MemoryStorageWithGas::default();

        assert_eq!(storage.gas_used.take(), StorageGasUsed::default());
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

    #[test]
    fn works_with_multi_test() {
        let storage = MemoryStorageWithGas::new();

        AppBuilder::new()
            .with_storage(&storage)
            .build(|r, _, storage| {
                r.bank
                    .init_balance(
                        storage,
                        &Addr::unchecked("admin"),
                        vec![Coin::new(100, "uluna")],
                    )
                    .unwrap();
            });

        let gas = storage.gas_used.borrow();
        assert_eq!(gas.last, 3650);
        assert_eq!(gas.write_cnt, 1);
    }
}
