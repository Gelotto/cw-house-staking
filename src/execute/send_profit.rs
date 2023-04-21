use crate::{
  models::{ContractResult, DelegationAccount},
  state::{DELEGATION_ACCOUNTS, NET_PROFIT, TOKEN},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_send_submsg;

pub fn send_profit(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let mut profit =
    if let Some(mut account) = DELEGATION_ACCOUNTS.may_load(deps.storage, info.sender.clone())? {
      account.take_profit(deps.storage, deps.api)?
    } else {
      Uint128::zero()
    };

  if DelegationAccount::get_count(deps.storage)? == Uint128::one() {
    NET_PROFIT.update(deps.storage, |dust| -> ContractResult<_> {
      profit += dust;
      Ok(Uint128::zero())
    })?;
  }

  let mut resp = Response::new().add_attributes(vec![
    attr("action", "send_profit"),
    attr("amount", profit.to_string()),
  ]);

  if !profit.is_zero() {
    resp = resp.add_submessage(build_send_submsg(
      &info.sender,
      profit,
      &TOKEN.load(deps.storage)?,
    )?);
  }

  Ok(resp)
}
