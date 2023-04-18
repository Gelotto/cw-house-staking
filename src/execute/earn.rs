use crate::{
  models::{ContractResult, Snapshot},
  state::{NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT_DELEGATION, SNAPSHOTS, SNAPSHOT_TICK},
  util::increment,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn earn(
  deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  income: Uint128,
) -> ContractResult<Response> {
  let net_growth_delegation = NET_GROWTH_DELEGATION.load(deps.storage)?;
  let net_profit_delegation = NET_PROFIT_DELEGATION.load(deps.storage)?;
  let net_delegation = net_growth_delegation + net_profit_delegation;

  let growth_delta = if !net_delegation.is_zero() {
    income.multiply_ratio(net_growth_delegation, net_delegation)
  } else {
    income
  };

  increment(deps.storage, &NET_LIQUIDITY, growth_delta)?;

  let did_update = if let Some((i_snapshot, mut snapshot)) = Snapshot::get_latest(deps.storage)? {
    if snapshot.tick == SNAPSHOT_TICK.load(deps.storage)? {
      snapshot.income += income;
      SNAPSHOTS.save(deps.storage, i_snapshot, &snapshot)?;
      true
    } else {
      false
    }
  } else {
    false
  };

  if !did_update {
    Snapshot::create(deps.storage, deps.api, income, Uint128::zero())?;
  }

  Ok(Response::new().add_attributes(vec![attr("action", "earn")]))
}
