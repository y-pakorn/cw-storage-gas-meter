# Simple Storage Gas Meter

<p align="center">
    <a href="https://crates.io/crates/cw-storage-gas-meter">
        <img alt="Crates.io" src="https://img.shields.io/crates/v/cw-storage-gas-meter">
    </a>
    <a href="https://docs.rs/cw-storage-gas-meter">
        <img alt="docs.rs" src="https://img.shields.io/docsrs/cw-storage-gas-meter">
    </a>
    <img alt="Crates.io" src="https://img.shields.io/crates/l/cw-storage-gas-meter">
</p>

A simple CosmWasm storage gas meter for estimating gas usage from kv store.

## Usage

### Unit Test

Use `MemoryStorageWithGas` instead of `MemoryStorage` or `MockStorage`.

```rust
// let mut storage = MockStorage::new();
let mut storage = MemoryStorageWithGas::new();
let map = Map::<u64, Vec<u8>>::new("0");

let data = b"hello";
map.save(&mut storage, 0, &data.to_vec())?;

let gas = storage.last_gas_used();
assert_eq!(gas, 2960);
```

### Multi Test

Instantiate `cw_multi_test::App` with `MemoryStorageWithGas` instead of `MemoryStorage` or `MockStorage`.

Due to the nature of `cosmwasm_std::Storage` trait, we cannot downcast the `dyn Storage` back to `MemoryStorage` directly.

So we pass the pointer to the storage as trait object instead and access the gas log through that pointer.

```rust
let storage = MemoryStorageWithGas::new();

AppBuilder::new()
    .with_storage(&storage) // <- ref ptr here
    .build(|r, _, storage| {
        r.bank
            .init_balance(
                storage,
                &Addr::unchecked("admin"),
                vec![Coin::new(100, "uluna")],
            )
            .unwrap();
    });

let gas = storage.last_gas_used();
assert_eq!(gas, 3650);
```
