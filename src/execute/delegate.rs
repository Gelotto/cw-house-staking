use crate::{
  models::{ContractResult, DelegationAccount, DelegationType},
  state::{amortize, MEMOIZATION_QUEUE, NET_LIQUIDITY, TOKEN},
  util::increment,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_from_msg, has_funds},
};

pub fn delegate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  growth_delegation: Uint128,
  profit_delegation: Uint128,
) -> ContractResult<Response> {
  let mut resp = Response::new().add_attributes(vec![attr("action", "stake")]);
  let total_delegation = growth_delegation + profit_delegation;

  if total_delegation.is_zero() {
    return Err(crate::error::ContractError::InsufficientDelegation {});
  }

  match TOKEN.load(deps.storage)? {
    Token::Native { denom } => {
      if !has_funds(&info.funds, total_delegation, &denom) {
        return Err(crate::error::ContractError::InsufficientFunds {});
      }
    },
    Token::Cw20 {
      address: cw20_address,
    } => {
      resp = resp.add_submessage(build_cw20_transfer_from_msg(
        &info.sender,
        &env.contract.address,
        &cw20_address,
        total_delegation,
      )?)
    },
  };

  let (account, is_new_account) =
    DelegationAccount::get_or_create(deps.storage, &info.sender, env.block.time)?;

  if is_new_account {
    MEMOIZATION_QUEUE.push_back(deps.storage, &info.sender)?;
  }

  if !growth_delegation.is_zero() {
    account.stake(
      deps.storage,
      deps.api,
      DelegationType::Growth,
      growth_delegation,
    )?;
  }

  if !profit_delegation.is_zero() {
    account.stake(
      deps.storage,
      deps.api,
      DelegationType::Profit,
      profit_delegation,
    )?;
  }

  increment(deps.storage, &NET_LIQUIDITY, total_delegation)?;

  amortize(deps.storage, deps.api, 5)?;

  Ok(resp)
}
