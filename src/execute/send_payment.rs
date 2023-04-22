use crate::{
  error::ContractError,
  models::{ContractResult, Snapshot},
  state::{amortize, CLIENT_ACCOUNTS, NET_LIQUIDITY, TOKEN},
  util::{decrement, validate_addr},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_send_submsg;

pub fn send_payment(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  raw_recipient: &String,
  payment: Uint128,
) -> ContractResult<Response> {
  let recipient = validate_addr(deps.api, raw_recipient)?;

  let resp = Response::new().add_attributes(vec![
    attr("action", "send_payment"),
    attr("amount", payment.to_string()),
    attr("recipient", recipient.to_string()),
  ]);

  if payment.is_zero() {
    return Ok(resp);
  }

  // update client data if exists or error:
  CLIENT_ACCOUNTS.update(
    deps.storage,
    info.sender.clone(),
    |maybe_client| -> ContractResult<_> {
      if let Some(mut client) = maybe_client {
        // tally client total historical payment amount sent
        client.liquidity_spent += payment;
        Ok(client)
      } else {
        Err(ContractError::NotAuthorized {})
      }
    },
  )?;

  // create a new delegation snapshot
  Snapshot::upsert(deps.storage, Uint128::zero(), payment)?;

  // remove payment amount from contract-level liquidity amount
  decrement(deps.storage, &NET_LIQUIDITY, payment)?;

  amortize(deps.storage, 5)?;

  // send response with token transfer submsg
  Ok(resp.add_submessage(build_send_submsg(
    &recipient,
    payment,
    &TOKEN.load(deps.storage)?,
  )?))
}
