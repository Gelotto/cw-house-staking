use crate::{
  models::ContractResult,
  state::{ACCOUNTS, SNAPSHOT_TICK},
  util::increment,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn take_profit(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let profit = if let Some(account) = ACCOUNTS.may_load(deps.storage, info.sender.clone())? {
    let profit = account.take_profit(deps.storage, deps.api)?;
    increment(deps.storage, &SNAPSHOT_TICK, Uint128::one())?;
    profit
  } else {
    Uint128::zero()
  };

  Ok(Response::new().add_attributes(vec![
    attr("action", "take_profit"),
    attr("amount", profit.to_string()),
  ]))
}
