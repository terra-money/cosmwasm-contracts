use cosmwasm_std::{StdResult, Uint128};
use cw20::{Cw20ReceiveMsg, Denom};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    //////////////////////////
    /// Creator Operations ///
    //////////////////////////
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

    ////////////////////////
    /// VestingAccount Operations ///
    ////////////////////////
    Claim {
        denoms: Vec<Denom>,
        recipient: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    VestingAccount {
        address: String,
        start_after: Option<Denom>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct VestingAccountResponse {
    pub address: String,
    pub vestings: Vec<VestingData>,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct VestingData {
    pub master_address: Option<String>,
    pub vesting_denom: Denom,
    pub vesting_amount: Uint128,
    pub vested_amount: Uint128,
    pub vesting_schedule: VestingSchedule,
    pub claimable_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VestingSchedule {
    /// LinearVesting is used to vest tokens linearly during a time period.
    /// The total_amount will be vested during this period.
    LinearVesting {
        start_time: String,      // vesting start time in second unit
        end_time: String,        // vesting end time in second unit
        vesting_amount: Uint128, // total vesting amount
    },
    /// PeriodicVesting is used to vest tokens
    /// at regular intervals for a specific period.
    /// To minimize calculation error,
    /// (end_time - start_time) should be multiple of vesting_interval
    /// deposit_amount = amount * ((end_time - start_time) / vesting_interval + 1)
    PeriodicVesting {
        start_time: String,       // vesting start time in second unit
        end_time: String,         // vesting end time in second unit
        vesting_interval: String, // vesting interval in second unit
        amount: Uint128,          // the amount will be vested in a interval
    },
}

impl VestingSchedule {
    pub fn vested_amount(&self, block_time: u64) -> StdResult<Uint128> {
        match self {
            VestingSchedule::LinearVesting {
                start_time,
                end_time,
                vesting_amount,
            } => {
                let start_time = start_time.parse::<u64>().unwrap();
                let end_time = end_time.parse::<u64>().unwrap();

                if block_time <= start_time {
                    return Ok(Uint128::zero());
                }

                if block_time >= end_time {
                    return Ok(*vesting_amount);
                }

                let vested_token = vesting_amount
                    .checked_mul(Uint128::from(block_time - start_time))?
                    .checked_div(Uint128::from(end_time - start_time))?;

                Ok(vested_token)
            }
            VestingSchedule::PeriodicVesting {
                start_time,
                end_time,
                vesting_interval,
                amount,
            } => {
                let start_time = start_time.parse::<u64>().unwrap();
                let end_time = end_time.parse::<u64>().unwrap();
                let vesting_interval = vesting_interval.parse::<u64>().unwrap();

                if block_time < start_time {
                    return Ok(Uint128::zero());
                }

                let num_interval = 1 + (end_time - start_time) / vesting_interval;
                if block_time >= end_time {
                    return Ok(amount.checked_mul(Uint128::from(num_interval))?);
                }

                let passed_interval = 1 + (block_time - start_time) / vesting_interval;
                Ok(amount.checked_mul(Uint128::from(passed_interval))?)
            }
        }
    }
}

#[test]
fn linear_vesting_vested_amount() {
    let schedule = VestingSchedule::LinearVesting {
        start_time: "100".to_string(),
        end_time: "110".to_string(),
        vesting_amount: Uint128::new(1000000u128),
    };

    assert_eq!(schedule.vested_amount(100).unwrap(), Uint128::zero());
    assert_eq!(
        schedule.vested_amount(105).unwrap(),
        Uint128::new(500000u128)
    );
    assert_eq!(
        schedule.vested_amount(110).unwrap(),
        Uint128::new(1000000u128)
    );
    assert_eq!(
        schedule.vested_amount(115).unwrap(),
        Uint128::new(1000000u128)
    );
}

#[test]
fn periodic_vesting_vested_amount() {
    let schedule = VestingSchedule::PeriodicVesting {
        start_time: "105".to_string(),
        end_time: "110".to_string(),
        vesting_interval: "5".to_string(),
        amount: Uint128::new(500000u128),
    };

    assert_eq!(schedule.vested_amount(100).unwrap(), Uint128::zero());
    assert_eq!(
        schedule.vested_amount(105).unwrap(),
        Uint128::new(500000u128)
    );
    assert_eq!(
        schedule.vested_amount(110).unwrap(),
        Uint128::new(1000000u128)
    );
    assert_eq!(
        schedule.vested_amount(115).unwrap(),
        Uint128::new(1000000u128)
    );
}
