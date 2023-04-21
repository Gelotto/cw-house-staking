use crate::{
  error::ContractError,
  models::{amortize, ContractResult, Snapshot},
  state::{CLIENT_ACCOUNTS, NET_LIQUIDITY, TOKEN},
  util::decrement,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_send_submsg;

pub fn send_payment(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  recipient: Addr,
  payment: Uint128,
) -> ContractResult<Response> {
  if payment.is_zero() {
    return Err(crate::error::ContractError::MissingAmount {});
  }

  // tally client total historical payment amount sent
  CLIENT_ACCOUNTS.update(
    deps.storage,
    info.sender.clone(),
    |maybe_client| -> ContractResult<_> {
      if let Some(mut client) = maybe_client {
        client.liquidity_spent += payment;
        Ok(client)
      } else {
        Err(ContractError::NotAuthorized {})
      }
    },
  )?;

  Snapshot::upsert(deps.storage, deps.api, Uint128::zero(), payment)?;

  decrement(deps.storage, &NET_LIQUIDITY, payment)?;

  amortize(deps.storage, deps.api, 5)?;

  Ok(
    Response::new()
      .add_attributes(vec![
        attr("action", "send_payment"),
        attr("amount", payment.to_string()),
        attr("recipient", recipient.to_string()),
      ])
      .add_submessage(build_send_submsg(
        &recipient,
        payment,
        &TOKEN.load(deps.storage)?,
      )?),
  )
}
