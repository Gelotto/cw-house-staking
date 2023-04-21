use crate::{
  error::ContractError,
  models::{ClientAccount, ContractResult},
  state::{is_allowed, CLIENT_ACCOUNTS, CLIENT_ACCOUNTS_LEN, NET_PCT_LIQUIDITY_ALLOCATED},
  util::increment,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn set_client(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  address: Addr,
  pct_liquidity: u32,
) -> ContractResult<Response> {
  if !is_allowed(&deps.as_ref(), &info.sender, "set_client")? {
    return Err(crate::error::ContractError::NotAuthorized {});
  }

  // NOTE: pct_liquidity is expressed in thousandths,
  // so 10 means 1%, 100 means 10%, etc.

  // create or update client account
  let mut prev_pct_liquidity: u32 = 0;

  // upsert a client account
  CLIENT_ACCOUNTS.update(
    deps.storage,
    info.sender.clone(),
    |maybe_account| -> ContractResult<_> {
      if let Some(mut account) = maybe_account {
        prev_pct_liquidity = account.pct_liquidity;
        account.pct_liquidity = pct_liquidity;
        Ok(account)
      } else {
        Ok(ClientAccount {
          owner: info.sender.clone(),
          pct_liquidity,
          created_at: env.block.time,
          revenue_generated: Uint128::zero(),
          liquidity_spent: Uint128::zero(),
        })
      }
    },
  )?;

  // increase the client account counter
  if prev_pct_liquidity == 0 {
    increment(deps.storage, &CLIENT_ACCOUNTS_LEN, 1)?;
  }

  // increase the net allocated liquidity percent
  NET_PCT_LIQUIDITY_ALLOCATED.update(deps.storage, |prev_pct_total| -> ContractResult<_> {
    let new_pct_total = prev_pct_total - prev_pct_liquidity + pct_liquidity;
    // if there's not enough liquidity pct remaining to satisfy the client
    // abort the request.
    if (new_pct_total) > 1000 {
      return Err(ContractError::InsufficientLiquidity {});
    }
    Ok(new_pct_total)
  })?;

  Ok(Response::new().add_attributes(vec![
    attr("action", "set_client"),
    attr("client_address", address.to_string()),
    attr("pct_liquidity", pct_liquidity.to_string()),
  ]))
}
