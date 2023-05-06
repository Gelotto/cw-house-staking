use crate::{
  models::DelegationType,
  msg::{AccountView, PoolsView, SelectResponse, StatsView},
  state::{
    CLIENT_ACCOUNTS, CLIENT_ACCOUNTS_LEN, DELEGATION_ACCOUNTS, DELEGATION_ACCOUNTS_LEN,
    NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT, NET_PROFIT_DELEGATION, SNAPSHOTS,
    SNAPSHOTS_LEN,
  },
};
use cosmwasm_std::{Addr, Deps, Order, StdResult, Uint128};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  maybe_fields: Option<Vec<String>>,
  maybe_wallet: Option<Addr>,
) -> StdResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &maybe_fields);
  Ok(SelectResponse {
    // total spendable liquidity available
    total_liquidity: loader.get("liquidity", &NET_LIQUIDITY)?,

    // total unclaimed profit stored on behalf of all delegators
    total_profit_claimable: loader.get("profit", &NET_PROFIT)?,

    // 20 most recent Snapshots in time-desc order
    snapshots: loader.view("snapshots", || {
      Ok(Some(
        SNAPSHOTS
          .range(deps.storage, None, None, Order::Descending)
          .map(|result| result.unwrap().1)
          .take(20)
          .collect(),
      ))
    })?,

    // total delegation amounts for both the revenue growth and profit pools
    pools: loader.view("pools", || {
      Ok(Some(PoolsView {
        growth: NET_GROWTH_DELEGATION.load(deps.storage)?,
        profit: NET_PROFIT_DELEGATION.load(deps.storage)?,
      }))
    })?,

    // top-level statistics
    stats: loader.view("stats", || {
      Ok(Some(StatsView {
        n_delegation_accounts: DELEGATION_ACCOUNTS_LEN.load(deps.storage)?,
        n_client_accounts: CLIENT_ACCOUNTS_LEN.load(deps.storage)?,
        n_snapshots: SNAPSHOTS_LEN.load(deps.storage)?,
      }))
    })?,

    // data associated with the given "wallet" address argument
    account: loader.view_by_wallet("account", maybe_wallet, |wallet| {
      let (mut growth, mut loss, mut profit) =
        match DELEGATION_ACCOUNTS.may_load(deps.storage, wallet.clone())? {
          Some(account) => {
            let (growth, loss) = account
              .claim_readonly(deps.storage, DelegationType::Growth)
              .unwrap_or((Uint128::zero(), Uint128::zero()));

            let profit = account
              .claim_readonly(deps.storage, DelegationType::Profit)
              .unwrap_or((Uint128::zero(), Uint128::zero()))
              .0;

            (growth, loss, profit)
          },
          None => (Uint128::zero(), Uint128::zero(), Uint128::zero()),
        };

      let mut growth_delegation = Uint128::zero();
      let mut profit_delegation = Uint128::zero();

      if let Some(account) = DELEGATION_ACCOUNTS.may_load(deps.storage, wallet.clone())? {
        growth += account.memoized_growth;
        loss += account.memoized_loss;
        profit += account.memoized_profit;
        if let Ok((growth_deleg, profit_deleg)) = account.get_delegation_amounts(deps.storage) {
          growth_delegation += growth_deleg;
          profit_delegation += profit_deleg;
        }
      }

      let (liquidity_spent, revenue_generated) =
        match CLIENT_ACCOUNTS.may_load(deps.storage, wallet.clone())? {
          Some(client) => (client.amount_spent, client.amount_received),
          None => (Uint128::zero(), Uint128::zero()),
        };

      Ok(Some(AccountView {
        growth_delegation,
        profit_delegation,
        liquidity_spent,
        revenue_generated,
        growth_claimable: growth,
        profit_claimable: profit,
        loss_claimable: loss,
      }))
    })?,
  })
}
