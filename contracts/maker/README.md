# Market Maker

Simple demo contract using the Terra custom bindings to interact with
the swap contract. This can be used as is, but is mainly designed to help
to integration tests on the cosmwasm-terra custom functionality, as well
as serve as a basis for more detailed contracts.

## Compilation

The suggest way to build an image is this (in the root directory):

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/contracts/maker/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.9.0 ./contracts/maker
```

This was used to produce `contract.wasm` and `hash.txt` in `contracts/maker`.
