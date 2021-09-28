## Token Swap

This contract is to provide interface for swapping a legacy token to a target token.

### Owner Operations

* Enable   - turn on swapping
* Disable  - turn off swapping
* Withdraw - withdraw all legacy & target token to given account address

### User Operations

* Swap - Transfer cw20 token to this contract using `Cw20ExecuteMsg::Send` with `Cw20HookMsg::Swap` as msg.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    // Swap legacy token to target token
    Swap { recipient: Option<String> },
}
```