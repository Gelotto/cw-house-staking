use crate::{
  error::ContractError,
  models::{amortize, ContractResult, Snapshot},
  state::{
    CLIENT_ACCOUNTS, NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT, NET_PROFIT_DELEGATION, TOKEN,
  },
  util::increment,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_from_msg, has_balance, has_funds},
};

pub fn receive_payment(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  payment: Uint128,
) -> ContractResult<Response> {
  if payment.is_zero() {
    return Err(crate::error::ContractError::MissingAmount {});
  }

  // tally client total historical payment amount received
  CLIENT_ACCOUNTS.update(
    deps.storage,
    info.sender.clone(),
    |maybe_client| -> ContractResult<_> {
      if let Some(mut client) = maybe_client {
        client.revenue_generated += payment;
        Ok(client)
      } else {
        Err(ContractError::NotAuthorized {})
      }
    },
  )?;

  let mut resp = Response::new().add_attributes(vec![attr("action", "receive_payment")]);

  // validate funding and add any required transfer submsg to response
  match TOKEN.load(deps.storage)? {
    Token::Native { denom } => {
      if !(has_funds(&info.funds, payment, &denom)
        && has_balance(deps.querier, &info.sender, payment, &denom, false)?)
      {
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
        payment,
      )?)
    },
  };

  let net_growth_delegation = NET_GROWTH_DELEGATION.load(deps.storage)?;
  let net_profit_delegation = NET_PROFIT_DELEGATION.load(deps.storage)?;
  let net_delegation = net_growth_delegation + net_profit_delegation;

  // increase NET_GROWTH_DELEGATION
  let growth_delta = if !net_delegation.is_zero() {
    payment.multiply_ratio(net_growth_delegation, net_delegation)
  } else {
    payment
  };
  if !growth_delta.is_zero() {
    increment(deps.storage, &NET_LIQUIDITY, growth_delta)?;
  }

  // increase NET_PROFIT_DELEGATION
  let profit_delta = payment - growth_delta;
  if !profit_delta.is_zero() {
    increment(deps.storage, &NET_PROFIT, profit_delta)?;
  }

  Snapshot::upsert(deps.storage, deps.api, payment, Uint128::zero())?;

  amortize(deps.storage, deps.api, 5)?;

  Ok(resp)
}
