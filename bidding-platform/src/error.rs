use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Too many/little native tokens sent")]
    TooManyOrLittleNativeTokensSent {},

    #[error("Wrong denom")]
    WrongDenom {},

    #[error("Bid is too low")]
    BidTooLow {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
