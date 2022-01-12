## Token Vesting

This contract is to provide LUNA vesting account feature and the vesting LUNA can be staked via Anchor Protocol.
The contract will be generated per a vesting account to separate staking rewards.

### Initiate Contract

When a initiator enable staking, the deposited LUNA will be converted into bLUNA via Anchor Hub Contract.

* disable staking
  ```json
  {
      "owner_address": "terra1~~",
      "enable_staking": false,
      "vesting_schedule": {
          "start_time": "16838388123",
          "end_time": "16838388133",
          "vesting_interval": "1", // vesting interval in second unit
          "vesting_ratio": "0.1" // deposit_amount * vesting_ratio tokens will be distributed per a interval. Given 0.1, then this schedule requires 10 times distribution
      }
  }
  ```
* enable staking
  ```json
  {
      "owner_address": "terra1~~",
      "enable_staking": true,
      // refer here: https://docs.anchorprotocol.com/smart-contracts/deployed-contracts#bluna-smart-contracts
      "staking_info": { 
          "bluna_token": "terra1~~",
          "hub_contract": "terra1~~",
          "reward_contract": "terra1~~",
          "validator": "terravaloper1~~"
      },
      "vesting_schedule": {
          "start_time": "16838388123",
          "end_time": "16838388126",
          "vesting_interval": "1", // vesting interval in second unit
          "vesting_ratio": "0.1" // deposit_amount * vesting_ratio tokens will be distributed per a interval, so 0.1 given then this schedule requires 10 intervals
      }
  }
  ```

### Vesting Account Operations

* ChangeOwner - change claim privileged account address to other address
* Claim - send newly vested token to the (`recipient` or `vesting_account`). The `claim_amount` is computed as (`vested_amount` - `claimed_amount`) and `claimed_amount` is updated to `vested_amount`.
* ClaimRewards - send bLUNA staking rewards to the given recipient address. This function only can be executed when `staking_enabled` is true

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ChangeOwner { new_owner: String },
    Claim { recipient: Option<String> },
    ClaimRewards { recipient: Option<String> },
}
```

### Deployed Contract CodeID

| columbus-5 | bombay-12 |
| ---------- | --------- |
| N/A        | 33465     |
