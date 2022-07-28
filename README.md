# Simple Storage Gas Meter

A simple gas meter for measuring estimate gas usage from kv store.

## Usage

### Unit Test

Use `MemoryStorageWithGas` instead of `MemoryStorage` or `MockStorage`.

```rust
// let mut storage = MockStorage::new();
let mut storage = MemoryStorageWithGas::new();
let map = Map::<u64, Vec<u8>>::new("0");

let data = b"hello";
map.save(&mut storage, 0, &data.to_vec())?;

let gas = storage.gas_used.borrow();
assert_eq!(gas.last, 2960);
assert_eq!(gas.write_cnt, 1);
```

### Multi Test

Instantiate `cw_multi_test::App` with `MemoryStorageWithGas` instead of `MemoryStorage` or `MockStorage`.

```rust
let app = AppBuilder::new()
    .with_storage(MemoryStorageWithGas::default())
    .build(|_, _, _| {});

// ...
```
