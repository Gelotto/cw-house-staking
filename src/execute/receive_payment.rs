use crate::{
  error::ContractError,
  models::{ContractResult, Snapshot},
  state::{
    amortize, CLIENT_ACCOUNTS, NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT,
    NET_PROFIT_DELEGATION, TOKEN,
  },
  util::increment,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_from_submsg, has_funds},
};

pub fn receive_payment(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  sender: Option<Addr>,
  amount: Uint128,
) -> ContractResult<Response> {
  let sender = sender.unwrap_or(info.sender.clone());
  let mut resp = Response::new().add_attributes(vec![attr("action", "receive_payment")]);

  deps.api.addr_validate(sender.as_str())?;

  if amount.is_zero() {
    return Ok(resp);
  }

  // tally client total historical payment amount received
  CLIENT_ACCOUNTS.update(
    deps.storage,
    info.sender.clone(),
    |maybe_client| -> ContractResult<_> {
      if let Some(mut client) = maybe_client {
        client.amount_received += amount;
        Ok(client)
      } else {
        Err(ContractError::NotAuthorized {})
      }
    },
  )?;

  // verify funding and add any necessary transfer submsg to response
  match TOKEN.load(deps.storage)? {
    Token::Native { denom } => {
      if !has_funds(&info.funds, amount, &denom) {
        return Err(crate::error::ContractError::InsufficientFunds {});
      }
    },
    Token::Cw20 {
      address: cw20_address,
    } => {
      resp = resp.add_submessage(build_cw20_transfer_from_submsg(
        &sender,
        &env.contract.address,
        &cw20_address,
        amount,
      )?)
    },
  };

  let net_growth_delegation = NET_GROWTH_DELEGATION.load(deps.storage)?;
  let net_profit_delegation = NET_PROFIT_DELEGATION.load(deps.storage)?;
  let net_delegation = net_growth_delegation + net_profit_delegation;

  // increase NET_LIQUIDITY
  let liquidity_delta = if !net_delegation.is_zero() {
    amount.multiply_ratio(net_growth_delegation, net_delegation)
  } else {
    amount
  };
  if !liquidity_delta.is_zero() {
    increment(deps.storage, &NET_LIQUIDITY, liquidity_delta)?;
  }

  // increase NET_PROFIT
  let profit_delta = amount.multiply_ratio(net_profit_delegation, net_delegation);
  if !profit_delta.is_zero() {
    increment(deps.storage, &NET_PROFIT, profit_delta)?;
  }

  // create a new delegation snapshot
  Snapshot::upsert(deps.storage, amount, Uint128::zero())?;

  amortize(deps.storage)?;

  Ok(resp)
}
