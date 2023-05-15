use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("SnapshotNotFound")]
  SnapshotNotFound {},

  #[error("InvalidAddress")]
  InvalidAddress {},

  #[error("NotFound")]
  NotFound {},

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("InsufficientFunds")]
  InsufficientFunds {},

  #[error("InsufficientLiquidity")]
  InsufficientLiquidity {},

  #[error("InsufficientDelegation")]
  InsufficientDelegation {},

  #[error("InsufficientAllowance")]
  InsufficientAllowance {},
}
