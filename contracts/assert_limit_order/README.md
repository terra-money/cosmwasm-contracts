# Assert Limit Order

This contract can be used to ensure that a `MsgSwap` results in the user receiving at least a specified amount of tokens, otherwise the whole transaction is cancelled. This is possible because transactions are considered atomic on Terra -- if one message in the transaction fails, the entire transaction is reversed (however the gas fees are still paid).

## Spec

If the following condition is not satisfied:

`(cur_balance - ask_prev_balance) > offer_amount * belief_price * (1 - slippage_tolerance)`

The entire transaction is aborted.

#### HandleMsg

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Check the current balance is increased as much as expected
    AssertLimitOrder {
        offer_amount: Uint128,
        ask_denom: String,
        ask_prev_balance: Uint128,
        belief_price: Decimal,
        slippage_tolerance: Decimal,
    },
}
```

## Usage

To use the Assert Limit Order contract, simply include a `MsgExecuteContract` AFTER your `MsgSwap` within the SAME transaction.

| Chain ID       | Contract Address                               |
| -------------- | ---------------------------------------------- |
| `columbus-4`   | `terra1zt4dwd7s4mxrtdjhz7q9tqrlykp0p4fqq987jf` |
| `tequila-0004` | `terra1q7cx44u3hk30pfz853catgx4v2x3aeltq3sklz` |

### Example

- Swap `1000 LUNA` to UST with `1%` slippage_tolerance
- Current Price: `2.6427 UST`
- Current UST Balance: `50 UST`

### Terra.js

```ts
import {
  LCDClient,
  MsgSwap,
  MsgExecuteContract,
  MnemonicKey,
  Coin,
} from "@terra-money/terra.js";

const assertLimitOrderContract = "terra1q7cx44u3hk30pfz853catgx4v2x3aeltq3sklz";

async function main(): Promise<void> {
  const terra = new LCDClient({
    chainID: "tequila-0004",
    URL: "https://tequila-lcd.terra.dev",
  });

  const mk = new MnemonicKey();
  const wallet = terra.wallet(mk);

  const offerCoin = new Coin("uluna", "1000000000");
  const askDenom = "uusd";

  // swap 1000 LUNA to UST
  const swap = new MsgSwap(mk.key.address, offerCoin, askDenom);

  // apply guard
  const assertLimitOrder = new MsgExecuteContract(
    mk.key.address,
    assertLimitOrderContract,
    {
      assert_limit_order: {
        offer_amount: offerCoin.amount.toString(),
        ask_denom: askDenom,
        ask_prev_balance: "50000000", // uusd balance prior (50 UST)
        belief_price: "2.6427",
        slippage_tolerance: "0.01",
      },
    }
  );

  const tx = await wallet.createAndSignTx({
    msgs: [swap, assertLimitOrder],
  });

  const txResult = await terra.tx.broadcast(tx);
}

main().catch(console.err());
```
