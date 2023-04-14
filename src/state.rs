use crate::models::{Account, ContractResult, Delegation};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::Snapshot};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub const OWNER: Item<Addr> = Item::new("owner");

pub const NET_REVENUE_DELEGATION: Item<Uint128> = Item::new("net_revenue_delegation");
pub const NET_REWARDS_DELEGATION: Item<Uint128> = Item::new("net_rewards_delegation");
pub const NET_REVENUE: Item<Uint128> = Item::new("net_revenue");
pub const NET_LOSS: Item<Uint128> = Item::new("net_loss");
pub const NET_REWARDS: Item<Uint128> = Item::new("net_rewards");

pub const SNAPSHOTS_SEQ_NO: Item<Uint128> = Item::new("snapshot_seq_no");
pub const SNAPSHOTS_LEN: Item<Uint128> = Item::new("snapshot_len");
pub const SNAPSHOTS: Map<u128, Snapshot> = Map::new("snapshots");

pub const REVENUE_DELEGATIONS_SEQ_NO: Map<Addr, u128> = Map::new("revenue_delegations_seq_no");
pub const REVENUE_DELEGATIONS: Map<(Addr, u128), Delegation> = Map::new("revenue_delegations");

pub const REWARDS_DELEGATIONS_SEQ_NO: Map<Addr, u128> = Map::new("rewards_delegations_seq_no");
pub const REWARDS_DELEGATIONS: Map<(Addr, u128), Delegation> = Map::new("rewards_delegations");

pub const ACCOUNTS: Map<Addr, Account> = Map::new("accounts");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  _env: &Env,
  info: &MessageInfo,
  _msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  NET_REVENUE_DELEGATION.save(deps.storage, &Uint128::zero())?;
  NET_REWARDS_DELEGATION.save(deps.storage, &Uint128::zero())?;
  NET_REVENUE.save(deps.storage, &Uint128::zero())?;
  NET_LOSS.save(deps.storage, &Uint128::zero())?;
  NET_REWARDS.save(deps.storage, &Uint128::zero())?;
  SNAPSHOTS_LEN.save(deps.storage, &Uint128::zero())?;
  SNAPSHOTS_SEQ_NO.save(deps.storage, &Uint128::zero())?;
  OWNER.save(deps.storage, &info.sender)?;
  Ok(())
}

pub fn is_owner(
  storage: &dyn Storage,
  addr: &Addr,
) -> StdResult<bool> {
  return Ok(OWNER.load(storage)? == *addr);
}

pub fn increment<T>(
  storage: &mut dyn Storage,
  item: &Item<T>,
  increment: T,
) -> ContractResult<T>
where
  T: DeserializeOwned + Serialize + std::ops::Add<Output = T>,
{
  item.update(storage, |x| -> ContractResult<_> { Ok(x + increment) })
}
