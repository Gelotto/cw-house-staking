use crate::{
  models::{ContractResult, DelegationAccount, DelegationType},
  state::{
    amortize, DELEGATION_ACCOUNTS, DELEGATION_ACCOUNTS_LEN, MEMOIZATION_QUEUE, NET_LIQUIDITY, TOKEN,
  },
  util::increment,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Storage, Timestamp, Uint128};
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

  // check payment amounts and add any necessary submsgs to response:
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

  let account = get_or_create_account(deps.storage, &info.sender, env.block.time)?;

  // create separate Delegation records for both delegation amounts
  if !growth_delegation.is_zero() {
    account.delegate(deps.storage, DelegationType::Growth, growth_delegation)?;
  }
  if !profit_delegation.is_zero() {
    account.delegate(deps.storage, DelegationType::Profit, profit_delegation)?;
  }

  // add total delegation to contract-level net liquidity accumulator
  increment(deps.storage, &NET_LIQUIDITY, total_delegation)?;

  amortize(deps.storage, 5)?;

  Ok(resp)
}

fn get_or_create_account(
  storage: &mut dyn Storage,
  owner: &Addr,
  created_at: Timestamp,
) -> ContractResult<DelegationAccount> {
  let mut is_new_account = false;
  let account = DELEGATION_ACCOUNTS.update(
    storage,
    owner.clone(),
    |maybe_account| -> ContractResult<_> {
      if let Some(account) = maybe_account {
        Ok(account)
      } else {
        is_new_account = true;
        Ok(DelegationAccount::new(owner, created_at))
      }
    },
  )?;

  if is_new_account {
    // adjust global DelegationAccount counter
    increment(storage, &DELEGATION_ACCOUNTS_LEN, Uint128::one())?;

    // add the new account to the back of the memoization queue for use
    // by amortization.
    MEMOIZATION_QUEUE.push_back(storage, owner)?;
  }

  Ok(account)
}
