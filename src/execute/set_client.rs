use crate::{
  models::{ClientAccount, ContractResult},
  state::{is_allowed, CLIENT_ACCOUNTS, CLIENT_ACCOUNTS_LEN},
  util::{increment, validate_addr},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn set_client(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  raw_address: &String,
) -> ContractResult<Response> {
  if !is_allowed(&deps.as_ref(), &info.sender, "set_client")? {
    return Err(crate::error::ContractError::NotAuthorized {});
  }

  let address = validate_addr(deps.api, raw_address)?;
  let mut is_new_account = false;

  // upsert a client account
  CLIENT_ACCOUNTS.update(
    deps.storage,
    address.clone(),
    |maybe_account| -> ContractResult<_> {
      if let Some(account) = maybe_account {
        Ok(account)
      } else {
        is_new_account = true;
        Ok(ClientAccount {
          owner: address.clone(),
          created_at: env.block.time,
          revenue_generated: Uint128::zero(),
          liquidity_spent: Uint128::zero(),
        })
      }
    },
  )?;

  // increase the client account counter
  if is_new_account {
    increment(deps.storage, &CLIENT_ACCOUNTS_LEN, 1)?;
  }

  Ok(Response::new().add_attributes(vec![
    attr("action", "set_client"),
    attr("client_address", raw_address.to_string()),
  ]))
}
