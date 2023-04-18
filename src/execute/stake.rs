use crate::{
  models::{Account, ContractResult, DelegationType},
  state::{NET_LIQUIDITY, SNAPSHOT_TICK},
  util::increment,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn stake(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  growth_delegation: Uint128,
  profit_delegation: Uint128,
) -> ContractResult<Response> {
  let account = Account::get_or_create(deps.storage, &info.sender)?;
  let mut total_delegation = Uint128::zero();

  if !growth_delegation.is_zero() {
    account.stake(
      deps.storage,
      deps.api,
      DelegationType::Growth,
      growth_delegation,
    )?;
    total_delegation += growth_delegation;
  }

  if !profit_delegation.is_zero() {
    account.stake(
      deps.storage,
      deps.api,
      DelegationType::Profit,
      profit_delegation,
    )?;
    total_delegation += profit_delegation;
  }

  increment(deps.storage, &NET_LIQUIDITY, total_delegation)?;
  increment(deps.storage, &SNAPSHOT_TICK, Uint128::one())?;

  Ok(Response::new().add_attributes(vec![attr("action", "stake")]))
}
