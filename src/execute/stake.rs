use crate::{
  models::{Account, ContractResult, DelegationType},
  state::{ACCOUNT_MEMOIZATION_QUEUE, GLTO_CW20_CONTRACT_ADDR, NET_LIQUIDITY},
  util::increment,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_cw20_transfer_from_msg;

pub fn stake(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  growth_delegation: Uint128,
  profit_delegation: Uint128,
) -> ContractResult<Response> {
  let (account, is_new_account) = Account::get_or_create(deps.storage, &info.sender)?;
  let mut total_delegation = Uint128::zero();

  if is_new_account {
    ACCOUNT_MEMOIZATION_QUEUE.push_back(deps.storage, &info.sender)?;
  }

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

  if total_delegation.is_zero() {
    return Err(crate::error::ContractError::MissingAmount {});
  }

  increment(deps.storage, &NET_LIQUIDITY, total_delegation)?;

  Account::amortize_claim_function(deps.storage, deps.api, 5)?;

  Ok(
    Response::new()
      .add_attributes(vec![attr("action", "stake")])
      .add_submessage(build_cw20_transfer_from_msg(
        &info.sender,
        &env.contract.address,
        &Addr::unchecked(GLTO_CW20_CONTRACT_ADDR),
        total_delegation,
      )?),
  )
}
