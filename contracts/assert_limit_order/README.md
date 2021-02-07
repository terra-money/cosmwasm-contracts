# Assert Limit Order

The contract is to ensure the receive amount when executing native `swap` operation. It must be used after actually `swap` message in the same transaction, then it will check the ask balance to assert slippage tolerance.

To avoid unnecessary tax charging, the contract itself does not receive or relay actual the swap msg.

Ex) 

* Swap `1 LUNA` to KRT with `1%` slippage_tolerance 
* Current Price: `2,700 KRT`
* KRT Balance: `1,000 KRT`

```
Tx {
    Msg {
        Swap {
            offer: {
                denom: 'uluna',
                amount: '1000000',
            },
            ask_denom: 'ukrw',
        }
    },
    Msg {
        ExecuteContract {
            AssertLimitOrder {
                'offer_amount': '1000000',
                'ask_denom': 'ukrw',
                'ask_prev_balance': '1000000000',
                'belief_price': '2700',
                'slippage_tolerance': '0.01',
            }
        }
    }
}
```

This will enforce the swaped amount (`cur_balance - ask_prev_balance`) must be bigger than 
`offer_amount * belief_price * (1 - slippage_tolerance)`.
