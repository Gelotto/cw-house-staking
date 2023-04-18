use crate::{
  msg::{DelegationTotals, SelectResponse},
  state::{NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT_DELEGATION},
};
use cosmwasm_std::{Addr, Deps, StdResult};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
  _wallet: Option<Addr>,
) -> StdResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &fields);
  Ok(SelectResponse {
    liquidity: loader.get("liquidity", &NET_LIQUIDITY)?,
    pools: loader.view("pools", || {
      Ok(Some(DelegationTotals {
        growth: NET_GROWTH_DELEGATION.load(deps.storage)?,
        profit: NET_PROFIT_DELEGATION.load(deps.storage)?,
      }))
    })?,
  })
}
