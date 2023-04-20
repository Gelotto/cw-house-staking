use crate::{
  models::{Account, ContractResult},
  state::{ACCOUNTS, GLTO_CW20_CONTRACT_ADDR, NET_PROFIT},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_cw20_transfer_msg;

pub fn take_profit(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let mut profit =
    if let Some(mut account) = ACCOUNTS.may_load(deps.storage, info.sender.clone())? {
      account.take_profit(deps.storage, deps.api)?
    } else {
      Uint128::zero()
    };

  if Account::get_count(deps.storage)? == Uint128::one() {
    NET_PROFIT.update(deps.storage, |dust| -> ContractResult<_> {
      profit += dust;
      Ok(Uint128::zero())
    })?;
  }

  let mut resp = Response::new().add_attributes(vec![
    attr("action", "take_profit"),
    attr("amount", profit.to_string()),
  ]);

  if !profit.is_zero() {
    resp = resp.add_submessage(build_cw20_transfer_msg(
      &info.sender,
      &Addr::unchecked(GLTO_CW20_CONTRACT_ADDR),
      profit,
    )?);
  }

  Ok(resp)
}
