
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std(#[from] StdError),
    // this is whatever we want
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Name is not in the expected format (3-30 UTF-8 bytes)")]
    InvalidName,

    #[error("Ticker symbol is not in expected format")]
    InvalidSymbol,

    #[error("Decimals must not exceed 18")]
    InvalidDecimals,

    #[error("Insufficient allowance")]
    InsufficientAllowance,

    #[error("Insufficient funds")]
    InsufficientFunds,
}