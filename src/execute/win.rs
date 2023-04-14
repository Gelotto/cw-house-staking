use crate::{
  models::{ContractResult, Snapshot},
  state::{increment, NET_REVENUE, NET_REVENUE_DELEGATION, NET_REWARDS, NET_REWARDS_DELEGATION},
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn win(
  deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  earnings: Uint128,
) -> ContractResult<Response> {
  let net_revenue_delegation = NET_REVENUE_DELEGATION.load(deps.storage)?;
  let net_rewards_delegation = NET_REWARDS_DELEGATION.load(deps.storage)?;
  let net_total_delegation = net_revenue_delegation + net_rewards_delegation;

  let revenue_delta = earnings.multiply_ratio(net_revenue_delegation, net_total_delegation);
  let rewards_delta = earnings - revenue_delta;

  increment(deps.storage, &NET_REVENUE, revenue_delta)?;
  increment(deps.storage, &NET_REWARDS, rewards_delta)?;

  Snapshot::create(deps.storage, earnings, Uint128::zero())?;

  Ok(Response::new().add_attributes(vec![attr("action", "win")]))
}
