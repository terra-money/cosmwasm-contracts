## Token Vesting

This contract is to provide vesting account feature for the both cw20 and native tokens.

### Master Operations

* RegisterVestingAccount   - register vesting account
  * When creating vesting account, the one can specify the `master_address` to enable deregister feature.
* DeregisterVestingAccount  - deregister vesting account
  * This interface only executable from the `master_address` of a vesting account.
  * It will compute `claimable_amount` and `left_vesting_amount`. Each amount respectively sent to (`vested_token_recipient` or `vesting_account`) and (`left_vesting_token_recipient` or `master_address`).

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterVestingAccount {
        master_address: Option<String>, // if given, the vesting account can be unregistered
        address: String,
        vesting_schedule: VestingSchedule,
    },
    /// only available when master_address was set
    DeregisterVestingAccount {
        address: String,
        denom: Denom,
        vested_token_recipient: Option<String>,
        left_vesting_token_recipient: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Register vesting account with token transfer
    RegisterVestingAccount {
        master_address: Option<String>, // if given, the vesting account can be unregistered
        address: String,
        vesting_schedule: VestingSchedule,
    },
}
```

### Vesting Account Operations

* Claim - send newly vested token to the (`recipient` or `vesting_account`). The `claim_amount` is computed as (`vested_amount` - `claimed_amount`) and `claimed_amount` is updated to `vested_amount`.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ////////////////////////
    /// VestingAccount Operations ///
    ////////////////////////
    Claim {
        denoms: Vec<Denom>,
        recipient: Option<String>,
    },
}
```

### Deployed Contract Info
| data          | bombay-12                                    | columbus-5 |
| ------------- | -------------------------------------------- | ---------- |
| code_id       | 35340                                        | N/A        |
| contract_addr | terra15uc49grd8h0xxj3jvmcx9yswvw8v0ypy32pe8m | N/A        |
