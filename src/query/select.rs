use crate::{
  msg::{Accounts, DelegationTotals, SelectResponse},
  state::{
    CLIENT_ACCOUNTS, DELEGATION_ACCOUNTS, NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT,
    NET_PROFIT_DELEGATION, SNAPSHOTS,
  },
};
use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  maybe_fields: Option<Vec<String>>,
  maybe_wallet: Option<Addr>,
) -> StdResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &maybe_fields);
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
    accounts: loader.view_by_wallet("accounts", maybe_wallet, |owner| {
      Ok(Some(Accounts {
        client: CLIENT_ACCOUNTS.may_load(deps.storage, owner.clone())?,
        delegation: DELEGATION_ACCOUNTS.may_load(deps.storage, owner.clone())?,
      }))
    })?,
  })
}
