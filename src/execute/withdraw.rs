use crate::{
  models::ContractResult,
  state::{ACCOUNTS, SNAPSHOT_TICK},
  util::increment,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn withdraw(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  let balance = if let Some(account) = ACCOUNTS.may_load(deps.storage, info.sender.clone())? {
    let balance = account.withdraw(deps.storage, deps.api)?;
    increment(deps.storage, &SNAPSHOT_TICK, Uint128::one())?;
    ACCOUNTS.remove(deps.storage, info.sender.clone());
    balance
  } else {
    Uint128::zero()
  };

  Ok(Response::new().add_attributes(vec![
    attr("action", "withdraw"),
    attr("amount", balance.to_string()),
  ]))
}
