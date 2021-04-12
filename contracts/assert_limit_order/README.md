# Assert Limit Order

This contract can be used to ensure that a `MsgSwap` results in the user receiving at least a specified amount of tokens, otherwise the whole transaction is cancelled. This is possible because transactions are considered atomic on Terra -- if one message in the transaction fails, the entire transaction is reversed (however the gas fees are still paid).

## Spec

If the following condition is not satisfied:

`SwapSimulation.receive < minimum_receive`

The entire transaction is aborted.

#### ExecuteMsg

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Check the current balance is increased as much as expected
    AssertLimitOrder {
        offer_amount: Coin,
        ask_denom: String,
        minimum_receive: Uint128,
    },
}
```

## Usage

To use the Assert Limit Order contract, simply include a `MsgExecuteContract` BEFORE your `MsgSwap` within the SAME transaction.

| Chain ID       | Contract Address                               |
| -------------- | ---------------------------------------------- |
| `columbus-4`   | `terra1vs9jr7pxuqwct3j29lez3pfetuu8xmq7tk3lzk` |
| `tequila-0004` | `terra1z3sf42ywpuhxdh78rr5vyqxpaxa0dx657x5trs` |

### Example

- Swap `1000 UST` to LUNA
- Minimum Receive: `374.616869 LUNA`

### Terra.js

```ts
import {
  LCDClient,
  MsgSwap,
  MsgExecuteContract,
  MnemonicKey,
  Coin,
} from "@terra-money/terra.js";

const assertLimitOrderContract = "terra1z3sf42ywpuhxdh78rr5vyqxpaxa0dx657x5trs";

async function main(): Promise<void> {
  const terra = new LCDClient({
    chainID: "tequila-0004",
    URL: "https://tequila-lcd.terra.dev",
  });

  const mk = new MnemonicKey();
  const wallet = terra.wallet(mk);

  const offerCoin = new Coin("uusd", "1000000000");
  const askDenom = "uluna";

  // swap 1000 LUNA to UST
  const swap = new MsgSwap(mk.key.address, offerCoin, askDenom);

  // apply guard
  const assertLimitOrder = new MsgExecuteContract(
    mk.key.address,
    assertLimitOrderContract,
    {
      assert_limit_order: {
        offer_coin: {
          denom: offerCoin.denom,
          amount: offerCoin.amount.toString(),
        },
        ask_denom: askDenom,
        minimum_receive: "374616869",
      },
    }
  );

  const tx = await wallet.createAndSignTx({
    msgs: [assertLimitOrder, swap],
  });

  const txResult = await terra.tx.broadcast(tx);
}

main().catch(console.err());
```
