use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_lib::models::{Owner, Token};

use crate::models::Snapshot;

#[cw_serde]
pub struct InstantiateMsg {
  pub owner: Owner,
  pub token: Token,
}

#[cw_serde]
pub enum ExecuteMsg {
  SetClient {
    address: Addr,
  },
  Delegate {
    growth: Uint128,
    profit: Uint128,
  },
  ReceivePayment {
    sender: Option<Addr>,
    amount: Uint128,
  },
  SendPayment {
    recipient: Addr,
    amount: Uint128,
  },
  SendProfit {},
  Withdraw {},
}

#[cw_serde]
pub enum QueryMsg {
  Select {
    fields: Option<Vec<String>>,
    wallet: Option<Addr>,
  },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct PoolsView {
  pub growth: Uint128,
  pub profit: Uint128,
}

#[cw_serde]
pub struct StatsView {
  pub n_delegation_accounts: u32,
  pub n_client_accounts: u32,
  pub n_snapshots: u32,
}

#[cw_serde]
pub struct AccountView {
  pub growth_delegation: Uint128,
  pub profit_delegation: Uint128,
  pub profit_claimable: Uint128,
  pub loss_claimable: Uint128,
  pub growth_claimable: Uint128,
  pub liquidity_spent: Uint128,
  pub revenue_generated: Uint128,
}

#[cw_serde]
pub struct SelectResponse {
  pub total_liquidity: Option<Uint128>,
  pub total_profit_claimable: Option<Uint128>,
  pub snapshots: Option<Vec<Snapshot>>,
  pub pools: Option<PoolsView>,
  pub account: Option<AccountView>,
  pub stats: Option<StatsView>,
}
