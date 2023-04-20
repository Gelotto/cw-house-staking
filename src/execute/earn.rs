use crate::{
  models::{Account, ContractResult, Snapshot},
  state::{
    GLTO_CW20_CONTRACT_ADDR, NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT,
    NET_PROFIT_DELEGATION,
  },
  util::increment,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_cw20_transfer_from_msg;

pub fn earn(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  income: Uint128,
) -> ContractResult<Response> {
  if income.is_zero() {
    return Err(crate::error::ContractError::MissingAmount {});
  }

  let net_growth_delegation = NET_GROWTH_DELEGATION.load(deps.storage)?;
  let net_profit_delegation = NET_PROFIT_DELEGATION.load(deps.storage)?;
  let net_delegation = net_growth_delegation + net_profit_delegation;

  let growth_delta = if !net_delegation.is_zero() {
    income.multiply_ratio(net_growth_delegation, net_delegation)
  } else {
    income
  };

  if !growth_delta.is_zero() {
    increment(deps.storage, &NET_LIQUIDITY, growth_delta)?;
  }

  let profit_delta = if !net_delegation.is_zero() {
    income.multiply_ratio(net_profit_delegation, net_delegation)
  } else {
    income
  };

  if !profit_delta.is_zero() {
    increment(deps.storage, &NET_PROFIT, profit_delta)?;
  }

  Snapshot::create(deps.storage, deps.api, income, Uint128::zero())?;
  Account::amortize_claim_function(deps.storage, deps.api, 5)?;

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "earn")])
      .add_submessage(build_cw20_transfer_from_msg(
        &info.sender,
        &env.contract.address,
        &Addr::unchecked(GLTO_CW20_CONTRACT_ADDR),
        income,
      )?),
  )
}
