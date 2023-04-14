use crate::models::{Account, ContractResult, Target};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn delegate(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  maybe_revenue_delegation: Option<Uint128>,
  maybe_rewards_delegation: Option<Uint128>,
) -> ContractResult<Response> {
  let account = Account::get_or_create(deps.storage, &info.sender)?;

  if let Some(amount) = maybe_revenue_delegation {
    account.increment_delegation(deps.storage, Target::Revenue, amount)?;
  }

  if let Some(amount) = maybe_rewards_delegation {
    account.increment_delegation(deps.storage, Target::Rewards, amount)?;
  }

  Ok(Response::new().add_attributes(vec![attr("action", "delegate")]))
}
