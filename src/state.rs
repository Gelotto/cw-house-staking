use std::collections::HashSet;

use crate::models::Snapshot;
use crate::models::{ClientAccount, ContractResult, Delegation, DelegationAccount};
use crate::msg::InstantiateMsg;
use crate::util::validate_addr;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Storage, Uint128};
use cw_acl::client::Acl;
use cw_lib::models::{Owner, Token};
use cw_storage_plus::{Deque, Item, Map};

pub const OWNER: Item<Owner> = Item::new("owner");
pub const TOKEN: Item<Token> = Item::new("token");

pub const NET_GROWTH_DELEGATION: Item<Uint128> = Item::new("net_growth_delegation");
pub const NET_PROFIT_DELEGATION: Item<Uint128> = Item::new("net_profit_delegation");
pub const NET_LIQUIDITY: Item<Uint128> = Item::new("net_liquidity");
pub const NET_PROFIT: Item<Uint128> = Item::new("net_profit");
pub const NET_PCT_LIQUIDITY_ALLOCATED: Item<u32> = Item::new("net_pct_liquidity_allocated");

pub const GROWTH_DELEGATOR_COUNT: Item<u32> = Item::new("growth_delegator_count");
pub const PROFIT_DELEGATOR_COUNT: Item<u32> = Item::new("profit_delegator_count");

pub const SNAPSHOTS: Map<u128, Snapshot> = Map::new("snapshots");
pub const SNAPSHOTS_LEN: Item<u32> = Item::new("snapshot_len");
pub const SNAPSHOTS_INDEX: Item<Uint128> = Item::new("snapshot_index");
pub const SNAPSHOT_SEQ_NO: Item<Uint128> = Item::new("snapshot_seq_no");

pub const GROWTH_DELEGATIONS: Map<(Addr, u128), Delegation> = Map::new("growth_delegations");
pub const GROWTH_DELEGATIONS_SEQ_NO: Map<Addr, u128> = Map::new("growth_delegations_seq_no");

pub const PROFIT_DELEGATIONS: Map<(Addr, u128), Delegation> = Map::new("profit_delegations");
pub const PROFIT_DELEGATIONS_SEQ_NO: Map<Addr, u128> = Map::new("profit_delegations_seq_no");

pub const DELEGATION_ACCOUNTS: Map<Addr, DelegationAccount> = Map::new("delegation_accounts");
pub const DELEGATION_ACCOUNTS_LEN: Item<u32> = Item::new("delegation_accounts_len");

pub const CLIENT_ACCOUNTS: Map<Addr, ClientAccount> = Map::new("client_accounts");
pub const CLIENT_ACCOUNTS_LEN: Item<u32> = Item::new("client_accounts_len");

pub const MEMOIZATION_QUEUE: Deque<Addr> = Deque::new("memoization_queue");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  _env: &Env,
  _info: &MessageInfo,
  msg: &InstantiateMsg,
) -> ContractResult<()> {
  validate_addr(
    deps.api,
    match &msg.owner {
      Owner::Address(addr) => addr,
      Owner::Acl(addr) => addr,
    },
  )?;

  OWNER.save(deps.storage, &msg.owner)?;
  TOKEN.save(deps.storage, &msg.token)?;
  NET_GROWTH_DELEGATION.save(deps.storage, &Uint128::zero())?;
  NET_PROFIT_DELEGATION.save(deps.storage, &Uint128::zero())?;
  NET_LIQUIDITY.save(deps.storage, &Uint128::zero())?;
  NET_PCT_LIQUIDITY_ALLOCATED.save(deps.storage, &0)?;
  NET_PROFIT.save(deps.storage, &Uint128::zero())?;
  DELEGATION_ACCOUNTS_LEN.save(deps.storage, &0)?;
  SNAPSHOTS_LEN.save(deps.storage, &0)?;
  SNAPSHOTS_INDEX.save(deps.storage, &Uint128::zero())?;
  SNAPSHOT_SEQ_NO.save(deps.storage, &Uint128::zero())?;
  GROWTH_DELEGATOR_COUNT.save(deps.storage, &0)?;
  PROFIT_DELEGATOR_COUNT.save(deps.storage, &0)?;
  CLIENT_ACCOUNTS_LEN.save(deps.storage, &0)?;

  Ok(())
}

/// Helper function that returns true if given wallet (principal) is authorized
/// by ACL to the given action.
pub fn is_allowed(
  deps: &Deps,
  principal: &Addr,
  action: &str,
) -> ContractResult<bool> {
  Ok(match OWNER.load(deps.storage)? {
    Owner::Address(addr) => *principal == addr,
    Owner::Acl(acl_addr) => {
      let acl = Acl::new(&acl_addr);
      acl.is_allowed(&deps.querier, principal, action)?
    },
  })
}

pub fn amortize(storage: &mut dyn Storage) -> ContractResult<()> {
  let n_accounts = 1;
  let n_retries = 2;
  let mut visited: HashSet<Addr> = HashSet::with_capacity(n_accounts as usize);
  for _ in 0..n_accounts {
    for _ in 0..n_retries {
      if let Some(owner) = MEMOIZATION_QUEUE.pop_front(storage)? {
        if visited.contains(&owner) {
          // already amorized all existing accounts
          MEMOIZATION_QUEUE.push_front(storage, &owner)?;
          return Ok(());
        }
        if let Some(mut account) = DELEGATION_ACCOUNTS.may_load(storage, owner.clone())? {
          account.memoize_claim_amounts(storage)?;
          visited.insert(owner.clone());
          MEMOIZATION_QUEUE.push_back(storage, &owner)?;
          DELEGATION_ACCOUNTS.save(storage, owner.clone(), &account)?;
        }
      } else {
        // queue is empty
        break;
      }
    }
  }
  Ok(())
}
