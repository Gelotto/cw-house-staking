use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Storage, Uint128};

use crate::{
  error::ContractError,
  state::{
    increment, ACCOUNTS, NET_REVENUE_DELEGATION, NET_REWARDS_DELEGATION, REVENUE_DELEGATIONS,
    REVENUE_DELEGATIONS_SEQ_NO, REWARDS_DELEGATIONS, REWARDS_DELEGATIONS_SEQ_NO, SNAPSHOTS,
    SNAPSHOTS_LEN, SNAPSHOTS_SEQ_NO,
  },
};

pub type ContractResult<T> = Result<T, ContractError>;

#[cw_serde]
pub enum Target {
  Revenue,
  Rewards,
}

#[cw_serde]
pub struct Account {
  pub owner: Addr,
}

#[cw_serde]
pub struct Snapshot {
  pub revenue_delegation: Uint128,
  pub rewards_delegation: Uint128,
  pub earnings: Uint128,
  pub loss: Uint128,
}

#[cw_serde]
pub struct Delegation {
  pub owner: Addr,
  pub amount: Uint128,
  pub i_snapshot: Uint128,
}

impl Snapshot {
  pub fn get_latest(storage: &mut dyn Storage) -> ContractResult<Option<(u128, Self)>> {
    if SNAPSHOTS_LEN.load(storage)?.is_zero() {
      return Ok(None);
    }
    let idx = SNAPSHOTS_SEQ_NO.load(storage)?.u128() - 1;
    Ok(Some((idx, SNAPSHOTS.load(storage, idx)?)))
  }

  pub fn get_latest_index(
    storage: &mut dyn Storage,
    increment: bool,
  ) -> ContractResult<Option<u128>> {
    if SNAPSHOTS_LEN.load(storage)?.is_zero() {
      return Ok(None);
    }
    if increment {
      Ok(Some(
        SNAPSHOTS_SEQ_NO
          .update(storage, |x| -> ContractResult<_> { Ok(x + Uint128::one()) })?
          .u128()
          - 1,
      ))
    } else {
      Ok(Some(SNAPSHOTS_SEQ_NO.load(storage)?.u128() - 1))
    }
  }

  pub fn create(
    storage: &mut dyn Storage,
    earnings: Uint128,
    loss: Uint128,
  ) -> ContractResult<Self> {
    let i_snapshot = Self::get_latest_index(storage, true)?.unwrap_or_default();
    let snapshot = Snapshot {
      revenue_delegation: NET_REVENUE_DELEGATION.load(storage)?,
      rewards_delegation: NET_REWARDS_DELEGATION.load(storage)?,
      earnings,
      loss,
    };
    increment(storage, &SNAPSHOTS_LEN, Uint128::one())?;
    SNAPSHOTS.save(storage, i_snapshot, &snapshot)?;
    Ok(snapshot)
  }
}

impl Delegation {}

impl Account {
  pub fn get_or_create(
    storage: &mut dyn Storage,
    owner: &Addr,
  ) -> ContractResult<Self> {
    ACCOUNTS.update(
      storage,
      owner.clone(),
      |maybe_account| -> ContractResult<_> {
        if let Some(account) = maybe_account {
          Ok(account)
        } else {
          Ok(Account {
            owner: owner.clone(),
          })
        }
      },
    )
  }

  pub fn get_latest_delegation(
    &self,
    storage: &mut dyn Storage,
    target: Target,
  ) -> ContractResult<Option<(u128, Delegation)>> {
    let (seq_no_item, delegations_map) = match target {
      Target::Revenue => (&REVENUE_DELEGATIONS_SEQ_NO, &REVENUE_DELEGATIONS),
      Target::Rewards => (&REWARDS_DELEGATIONS_SEQ_NO, &REWARDS_DELEGATIONS),
    };
    if let Some(idx) = seq_no_item.may_load(storage, self.owner.clone())? {
      if let Some(deleg) = delegations_map.may_load(storage, (self.owner.clone(), idx))? {
        return Ok(Some((idx, deleg)));
      } else {
        return Ok(None);
      }
    }
    Ok(None)
  }

  pub fn increment_delegation(
    &self,
    storage: &mut dyn Storage,
    target: Target,
    delta: Uint128,
  ) -> ContractResult<Uint128> {
    let (net_delegation_item, delegations_map, delegations_seq_no) = match target {
      Target::Revenue => (
        &NET_REVENUE_DELEGATION,
        &REVENUE_DELEGATIONS,
        &REVENUE_DELEGATIONS_SEQ_NO,
      ),
      Target::Rewards => (
        &NET_REWARDS_DELEGATION,
        &REWARDS_DELEGATIONS,
        &REWARDS_DELEGATIONS_SEQ_NO,
      ),
    };

    increment(storage, net_delegation_item, delta)?;

    let mut amount = delta.clone(); // new total delegation amount for the user
    let mut i_next_deleg: u128 = 0;

    // if no new snapshots have been made since the last time the user updated their delegation
    // simply increment the most recent past delegation created the user instead of creating
    // an entirely new one.
    if let Some((i_prev_deleg, mut prev_deleg)) = self.get_latest_delegation(storage, target)? {
      // set the new delegation amount to the previous amount plus the delta
      amount += prev_deleg.amount;
      if let Some(i_snapshot) = Snapshot::get_latest_index(storage, false)? {
        if i_snapshot == prev_deleg.i_snapshot.into() {
          prev_deleg.amount = amount;
          delegations_map.save(storage, (self.owner.clone(), i_prev_deleg), &prev_deleg)?;
          return Ok(prev_deleg.amount);
        }
      }
      // since we need to create a new Delegation, calculate its index for use
      // in its delegations map key:
      i_next_deleg = i_prev_deleg + 1;
    }

    // get the index of the next Snapshot to be made in the future
    let i_snapshot = SNAPSHOTS_SEQ_NO.load(storage)?;

    // increment the delegation sequence number
    delegations_seq_no.save(storage, self.owner.clone(), &i_next_deleg)?;

    // insert the new Delegation
    delegations_map.save(
      storage,
      (self.owner.clone(), i_next_deleg),
      &Delegation {
        owner: self.owner.clone(),
        amount,
        i_snapshot,
      },
    )?;

    Ok(amount)
  }
}
