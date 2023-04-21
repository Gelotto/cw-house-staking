use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_lib::models::Token;

use crate::models::{ClientAccount, DelegationAccount, Snapshot};

#[cw_serde]
pub struct InstantiateMsg {
  pub acl_address: Addr,
  pub token: Token,
}

#[cw_serde]
pub enum ExecuteMsg {
  SetClient { address: Addr },
  Delegate { growth: Uint128, profit: Uint128 },
  ReceivePayment { amount: Uint128 },
  SendPayment { recipient: Addr, amount: Uint128 },
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
pub struct DelegationTotals {
  pub growth: Uint128,
  pub profit: Uint128,
}

#[cw_serde]
pub struct Accounts {
  pub delegation: Option<DelegationAccount>,
  pub client: Option<ClientAccount>,
}

#[cw_serde]
pub struct SelectResponse {
  pub liquidity: Option<Uint128>,
  pub profit: Option<Uint128>,
  pub snapshots: Option<Vec<Snapshot>>,
  pub pools: Option<DelegationTotals>,
  pub accounts: Option<Accounts>,
}
