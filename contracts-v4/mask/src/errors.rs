
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MaskError {
    #[error("{0}")]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std(#[from] StdError),
    // this is whatever we want
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Messages empty. Must reflect at least one message")]
    MessagesEmpty,
}