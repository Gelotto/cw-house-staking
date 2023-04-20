use crate::models::{Account, Delegation};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::Snapshot};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Uint128};
use cw_storage_plus::{Deque, Item, Map};

// pub const GLTO_CW20_CONTRACT_ADDR: &str =
//   "juno1j0a9ymgngasfn3l5me8qpd53l5zlm9wurfdk7r65s5mg6tkxal3qpgf5se";
pub const GLTO_CW20_CONTRACT_ADDR: &str =
  "juno12v2yh574zf3t2wuaaec45trypxfgst3zf9hdm57zhzvcu9a0x3ts27qcah";

pub const NET_GROWTH_DELEGATION: Item<Uint128> = Item::new("net_growth_delegation");
pub const NET_PROFIT_DELEGATION: Item<Uint128> = Item::new("net_profit_delegation");
pub const NET_LIQUIDITY: Item<Uint128> = Item::new("net_liquidity");
pub const NET_PROFIT: Item<Uint128> = Item::new("net_profit");

pub const GROWTH_DELEGATOR_COUNT: Item<u32> = Item::new("growth_delegator_count");
pub const PROFIT_DELEGATOR_COUNT: Item<u32> = Item::new("profit_delegator_count");

pub const SNAPSHOTS_INDEX: Item<Uint128> = Item::new("snapshot_index");
pub const SNAPSHOTS_LEN: Item<Uint128> = Item::new("snapshot_len");
pub const SNAPSHOT_SEQ_NO: Item<Uint128> = Item::new("snapshot_seq_no");
pub const SNAPSHOTS: Map<u128, Snapshot> = Map::new("snapshots");

pub const GROWTH_DELEGATIONS_SEQ_NO: Map<Addr, u128> = Map::new("growth_delegations_seq_no");
pub const GROWTH_DELEGATIONS: Map<(Addr, u128), Delegation> = Map::new("growth_delegations");

pub const PROFIT_DELEGATIONS_SEQ_NO: Map<Addr, u128> = Map::new("profit_delegations_seq_no");
pub const PROFIT_DELEGATIONS: Map<(Addr, u128), Delegation> = Map::new("profit_delegations");

pub const ACCOUNTS: Map<Addr, Account> = Map::new("accounts");
pub const ACCOUNTS_LEN: Item<Uint128> = Item::new("accounts_len");
pub const ACCOUNT_MEMOIZATION_QUEUE: Deque<Addr> = Deque::new("account_memoization_queue");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  _env: &Env,
  _info: &MessageInfo,
  _msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  NET_GROWTH_DELEGATION.save(deps.storage, &Uint128::zero())?;
  NET_PROFIT_DELEGATION.save(deps.storage, &Uint128::zero())?;
  NET_LIQUIDITY.save(deps.storage, &Uint128::zero())?;
  NET_PROFIT.save(deps.storage, &Uint128::zero())?;
  ACCOUNTS_LEN.save(deps.storage, &Uint128::zero())?;
  SNAPSHOTS_LEN.save(deps.storage, &Uint128::zero())?;
  SNAPSHOTS_INDEX.save(deps.storage, &Uint128::zero())?;
  SNAPSHOT_SEQ_NO.save(deps.storage, &Uint128::zero())?;
  GROWTH_DELEGATOR_COUNT.save(deps.storage, &0)?;
  PROFIT_DELEGATOR_COUNT.save(deps.storage, &0)?;
  Ok(())
}
