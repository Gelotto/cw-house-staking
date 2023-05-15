use crate::{
  error::ContractError, models::ContractResult, msg::ClientResponse, state::CLIENT_ACCOUNTS,
  util::validate_addr,
};
use cosmwasm_std::{Addr, Deps};

pub fn get_client(
  deps: Deps,
  client_address: Addr,
) -> ContractResult<ClientResponse> {
  validate_addr(deps.api, &client_address)?;
  if let Some(client) = CLIENT_ACCOUNTS.may_load(deps.storage, client_address.clone())? {
    Ok(ClientResponse { client })
  } else {
    Err(ContractError::NotFound {})
  }
}
