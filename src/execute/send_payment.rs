use crate::{
  error::ContractError,
  models::{ContractResult, Snapshot},
  state::{amortize, CLIENT_ACCOUNTS, NET_LIQUIDITY, TOKEN},
  util::{decrement, mul_pct},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_send_submsg;

pub fn send_payment(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  recipient: Addr,
  payment: Uint128,
) -> ContractResult<Response> {
  let resp = Response::new().add_attributes(vec![
    attr("action", "send_payment"),
    attr("amount", payment.to_string()),
    attr("recipient", recipient.to_string()),
  ]);

  if payment.is_zero() {
    return Ok(resp);
  }

  let total_liquidity = NET_LIQUIDITY.load(deps.storage)?;

  // update client data if exists or error:
  CLIENT_ACCOUNTS.update(
    deps.storage,
    info.sender.clone(),
    |maybe_client| -> ContractResult<_> {
      if let Some(mut client) = maybe_client {
        // make sure that the client's payment amount does exceed their max
        // liquidity utilization percentage.
        let allowance = mul_pct(total_liquidity, client.pct_liquidity.into());
        if payment > allowance {
          return Err(ContractError::InsufficientAllowance {});
        }
        // tally client total historical payment amount sent
        client.liquidity_spent += payment;
        Ok(client)
      } else {
        Err(ContractError::NotAuthorized {})
      }
    },
  )?;

  Snapshot::upsert(deps.storage, deps.api, Uint128::zero(), payment)?;

  decrement(deps.storage, &NET_LIQUIDITY, payment)?;

  amortize(deps.storage, deps.api, 5)?;

  Ok(resp.add_submessage(build_send_submsg(
    &recipient,
    payment,
    &TOKEN.load(deps.storage)?,
  )?))
}
