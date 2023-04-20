use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("SnapshotNotFound")]
  SnapshotNotFound {},

  #[error("NotAuthorized")]
  NotAuthorized {},

  #[error("ValidationError")]
  ValidationError {},

  #[error("MissingAmount")]
  MissingAmount {},
}
