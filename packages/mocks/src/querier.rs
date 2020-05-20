use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{Coin, Decimal, FullDelegation, HumanAddr, Uint128, Validator};

use crate::{OracleQuerier, SwapQuerier, TreasuryQuerier};

pub struct TerraMockQuerier {
    base: MockQuerier,
    swap: SwapQuerier,
    oracle: OracleQuerier,
    treasury: TreasuryQuerier,
}

impl TerraMockQuerier {
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        TerraMockQuerier {
            base: MockQuerier::new(balances),
            swap: SwapQuerier::default(),
            oracle: OracleQuerier::default(),
            treasury: TreasuryQuerier::default(),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance<U: Into<HumanAddr>>(
        &mut self,
        addr: U,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.base.update_balance(addr, balance)
    }

    // configure the stacking mock querier
    pub fn with_staking(
        &mut self,
        denom: &str,
        validators: &[Validator],
        delegations: &[FullDelegation],
    ) {
        self.base.with_staking(denom, validators, delegations)
    }

    pub fn with_market(&mut self, rates: &[(&str, &str, Decimal)], taxes: &[(&str, Decimal)]) {
        self.oracle = OracleQuerier::new(rates, taxes);
        self.swap = SwapQuerier::new(rates);
    }

    pub fn with_treasury(
        &mut self,
        tax_rate: Decimal,
        tax_proceeds: &[Coin],
        tax_caps: &[(String, Uint128)],
        reward_rate: Decimal,
        seigniorage_proceeds: Uint128,
    ) {
        self.treasury = TreasuryQuerier::new(
            tax_rate,
            tax_proceeds,
            tax_caps,
            reward_rate,
            seigniorage_proceeds,
        );
    }
}
