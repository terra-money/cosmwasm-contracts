# Roll Staking Contract

The contract is for the purpose of distributing Terra native tokens(`ukrw`, `uusd`, `usdr`, `umnt`) to a dedicated token `T` stakers. The rewards are paid in proportion to the balance of `T` stakers. It introduces `roll` concept to reduce distribution cost. In the `roll` concept, the amount deposited by the user is divided and managed by `roll_unit`, and each dividen unit is called a `roll` (the remainder is ignored at distribution). 

It also introduces `deposit period` to simplify withdraw process, so each `roll` can receive rewards after `deposit period`.

## Features

* Deposit

    Any user can deposit `T` to this contract. Stakers must do perform `allow` operation to dedicated token contract first before exeucting `deposit` operation. The roll is automatically created when the cumulated balance exceeds `roll_unit`. Each created `roll` will be included in reward target after `deposit period`.

    ```json
    { "deposit": { "amount": "1000000" } }
    ```

    The rolls are created according to the accumulated balance.

    ```rust
    let before_number_of_roll: u32 = (staker.balance.u128() / config.roll_unit.u128())
        .try_into()
        .unwrap();

    let after_number_of_roll: u32 = ((staker.balance.u128() + amount.u128())
        / config.roll_unit.u128())
    .try_into()
    .unwrap();

    for i in before_number_of_roll..after_number_of_roll {
        roll_store(&mut deps.storage).set(
            &[staker_addr_raw.as_slice(), &i.to_be_bytes()].concat(),
            &to_vec(&RollState {
                owner: staker_addr_raw.clone(),
                creation_time: env.block.time,
            })?,
        );
    }
    ```

* Withdraw

    Stakers can withdraw whenever they want without any delay, and this can cause removal of `roll`s of the staker. 

    ```json
    { "withdraw": { "amount": "1000000" } }
    ```

    The rolls are removed according to the left balance.

    ```rust
    let before_number_of_roll: u32 = (staker.balance.u128() / config.roll_unit.u128())
            .try_into()
            .unwrap();

    let after_number_of_roll: u32 = ((staker.balance.u128() - amount.u128())
        / config.roll_unit.u128())
    .try_into()
    .unwrap();

    // remove rolls in DESC(index) order
    for i in after_number_of_roll..before_number_of_roll {
        roll_store(&mut deps.storage)
            .remove(&[staker_addr_raw.as_slice(), &i.to_be_bytes()].concat());
    }

    let balance: Uint128 = (staker.balance - amount)?;
    let number_of_rolls: Uint128 = Uint128(balance.u128() / config.roll_unit.u128());
    staker_store(&mut deps.storage).set(
        staker_addr_raw.as_slice(),
        &to_vec(&StakerState { balance, ..staker })?,
    );
    ```
* Claim

    Stakers can claim the collected rewards, which is stored in `staker` state. **It will cause to lose some amount of tokens due to tax charging on Terra network.**

    ```json
    { "claim": {}}
    ```

* Distribute

    Only the contract owner can execute `distribute` operation. It will distribute specified amount of rewards token to stakers. It means before executing `distribute` opertion, a owner must send rewards token to this contract manually or throw other contract operation.
    
    Due to the limation of floating point data type, the distribution amount can be left. 

    ```json
    { "distribute": {
        "amount": "1000000"
    }}
    ```

## Compilation

The suggest way to build an image is this (in the root directory):

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/contracts/roll_staking/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.9.0 ./contracts/roll_staking
```

This was used to produce `contract.wasm` and `hash.txt` in `contracts/maker`.