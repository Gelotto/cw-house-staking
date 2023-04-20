use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::models::Snapshot;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
  Stake { growth: Uint128, profit: Uint128 },
  TakeProfit {},
  Withdraw {},
  Earn { amount: Uint128 },
  Pay { recipient: Addr, amount: Uint128 },
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
pub struct DelegationTotals {
  pub growth: Uint128,
  pub profit: Uint128,
}

#[cw_serde]
pub struct SelectResponse {
  pub liquidity: Option<Uint128>,
  pub profit: Option<Uint128>,
  pub snapshots: Option<Vec<Snapshot>>,
  pub pools: Option<DelegationTotals>,
}
