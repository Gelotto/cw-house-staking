use crate::{
  models::ContractResult,
  state::{ACCOUNTS, GLTO_CW20_CONTRACT_ADDR},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_cw20_transfer_msg;

pub fn withdraw(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let amount = if let Some(account) = ACCOUNTS.may_load(deps.storage, info.sender.clone())? {
    let amount = account.withdraw(deps.storage, deps.api)?;
    ACCOUNTS.remove(deps.storage, info.sender.clone());
    amount
  } else {
    Uint128::zero()
  };

  let mut resp = Response::new().add_attributes(vec![
    attr("action", "withdraw"),
    attr("amount", amount.to_string()),
  ]);

  if !amount.is_zero() {
    resp = resp.add_submessage(build_cw20_transfer_msg(
      &info.sender,
      &Addr::unchecked(GLTO_CW20_CONTRACT_ADDR),
      amount,
    )?);
  }

  Ok(resp)
}
