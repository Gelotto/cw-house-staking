use crate::{
  msg::{DelegationTotals, SelectResponse},
  state::{NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT, NET_PROFIT_DELEGATION, SNAPSHOTS},
};
use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
  _wallet: Option<Addr>,
) -> StdResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &fields);
  Ok(SelectResponse {
    liquidity: loader.get("liquidity", &NET_LIQUIDITY)?,
    profit: loader.get("profit", &NET_PROFIT)?,
    snapshots: loader.view("snapshots", || {
      Ok(Some(
        SNAPSHOTS
          .range(deps.storage, None, None, Order::Ascending)
          .map(|result| result.unwrap().1)
          .collect(),
      ))
    })?,
    pools: loader.view("pools", || {
      Ok(Some(DelegationTotals {
        growth: NET_GROWTH_DELEGATION.load(deps.storage)?,
        profit: NET_PROFIT_DELEGATION.load(deps.storage)?,
      }))
    })?,
  })
}
