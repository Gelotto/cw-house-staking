use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Order, Storage, Timestamp, Uint128};
use cw_storage_plus::Map;

use crate::{
  error::ContractError,
  state::{
    DELEGATION_ACCOUNTS, DELEGATION_ACCOUNTS_LEN, GROWTH_DELEGATIONS, GROWTH_DELEGATIONS_SEQ_NO,
    GROWTH_DELEGATOR_COUNT, NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT,
    NET_PROFIT_DELEGATION, PROFIT_DELEGATIONS, PROFIT_DELEGATIONS_SEQ_NO, PROFIT_DELEGATOR_COUNT,
    SNAPSHOTS, SNAPSHOTS_INDEX, SNAPSHOTS_LEN, SNAPSHOT_SEQ_NO,
  },
  util::{decrement, increment},
};

pub type ContractResult<T> = Result<T, ContractError>;

#[cw_serde]
pub enum DelegationType {
  Growth,
  Profit,
}

#[cw_serde]
pub struct DelegationAccount {
  pub owner: Addr,
  pub created_at: Timestamp,
  pub memoized_profit: Uint128,
  pub memoized_growth: Uint128,
  pub memoized_loss: Uint128,
}

#[cw_serde]
pub struct ClientAccount {
  pub owner: Addr,
  pub created_at: Timestamp,
  pub amount_spent: Uint128,
  pub amount_received: Uint128,
}

#[cw_serde]
pub struct Snapshot {
  pub seq_no: Uint128,
  pub claims_remaining: u32,
  pub growth_delegation: Uint128,
  pub profit_delegation: Uint128,
  pub growth: Uint128,
  pub loss: Uint128,
}

#[cw_serde]
pub struct Delegation {
  pub owner: Addr,
  pub amount: Uint128,
  pub i_snapshot: Uint128,
}

impl Delegation {}

impl DelegationAccount {
  pub fn new(
    owner: &Addr,
    created_at: Timestamp,
  ) -> Self {
    Self {
      owner: owner.clone(),
      created_at,
      memoized_growth: Uint128::zero(),
      memoized_loss: Uint128::zero(),
      memoized_profit: Uint128::zero(),
    }
  }

  pub fn get_count(storage: &dyn Storage) -> ContractResult<u32> {
    Ok(DELEGATION_ACCOUNTS_LEN.load(storage)?)
  }

  pub fn has_delegation(
    &self,
    storage: &dyn Storage,
    target: DelegationType,
  ) -> ContractResult<bool> {
    let seq_no_item = match target {
      DelegationType::Growth => &GROWTH_DELEGATIONS_SEQ_NO,
      DelegationType::Profit => &PROFIT_DELEGATIONS_SEQ_NO,
    };
    Ok(seq_no_item.may_load(storage, self.owner.clone())?.is_some())
  }

  pub fn get_latest_delegation(
    &self,
    storage: &dyn Storage,
    target: DelegationType,
  ) -> ContractResult<Option<(u128, Delegation)>> {
    let (seq_no_item, delegations_map) = match target {
      DelegationType::Growth => (&GROWTH_DELEGATIONS_SEQ_NO, &GROWTH_DELEGATIONS),
      DelegationType::Profit => (&PROFIT_DELEGATIONS_SEQ_NO, &PROFIT_DELEGATIONS),
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

  pub fn delegate(
    &self,
    storage: &mut dyn Storage,
    target: DelegationType,
    delta: Uint128,
  ) -> ContractResult<Uint128> {
    let (net_delegation_item, delegations_map, delegations_seq_no, delegator_count_item) =
      match target {
        DelegationType::Growth => (
          &NET_GROWTH_DELEGATION,
          &GROWTH_DELEGATIONS,
          &GROWTH_DELEGATIONS_SEQ_NO,
          &GROWTH_DELEGATOR_COUNT,
        ),
        DelegationType::Profit => (
          &NET_PROFIT_DELEGATION,
          &PROFIT_DELEGATIONS,
          &PROFIT_DELEGATIONS_SEQ_NO,
          &PROFIT_DELEGATOR_COUNT,
        ),
      };

    increment(storage, net_delegation_item, delta)?;
    increment(storage, &SNAPSHOT_SEQ_NO, Uint128::one())?;

    let mut amount = delta.clone(); // new total delegation amount for the user
    let mut i_next_deleg: u128 = 0;

    // get the index of the next Snapshot to be made in the future
    let i_next_snapshot = Snapshot::get_next_index(storage)?;

    // if no new snapshots have been made since the last time the user updated their delegation
    // simply increment the most recent past delegation created the user instead of creating
    // an entirely new one.
    if let Some((i_prev_deleg, mut prev_deleg)) = self.get_latest_delegation(storage, target)? {
      // set the new delegation amount to the previous amount plus the delta
      amount += prev_deleg.amount;

      if i_next_snapshot == prev_deleg.i_snapshot.into() {
        prev_deleg.amount = amount;
        delegations_map.save(storage, (self.owner.clone(), i_prev_deleg), &prev_deleg)?;
        return Ok(prev_deleg.amount);
      }
      // otherwise...
      // since we need to create a new Delegation, calculate its index for use
      // in its delegations map key:
      i_next_deleg = i_prev_deleg + 1;
    } else {
      increment(storage, delegator_count_item, 1)?;
    }

    // increment the delegation sequence number
    delegations_seq_no.save(storage, self.owner.clone(), &i_next_deleg)?;

    // insert the new Delegation
    delegations_map.save(
      storage,
      (self.owner.clone(), i_next_deleg),
      &Delegation {
        owner: self.owner.clone(),
        amount,
        i_snapshot: i_next_snapshot.into(),
      },
    )?;

    Ok(amount)
  }

  pub fn send_profit(
    &mut self,
    storage: &mut dyn Storage,
  ) -> ContractResult<Uint128> {
    // add memoized profit to total profit and clear the memoized value.
    let mut amount = self.claim(storage, DelegationType::Profit, false)?.0 + self.memoized_profit;

    self.memoized_profit = Uint128::zero();

    // if anything was actually claimed, subtract it from the net profit accumulator
    if !amount.is_zero() {
      NET_PROFIT.update(storage, |x| -> ContractResult<_> {
        amount = x.min(amount);
        Ok(x - amount)
      })?;
    }

    // increase the snapshot seq no to signal that a new snapshot should be taken
    // the next time the house sends or receives payment
    increment(storage, &SNAPSHOT_SEQ_NO, Uint128::one())?;

    // save changes made to this account so far
    DELEGATION_ACCOUNTS.save(storage, self.owner.clone(), &self)?;

    Ok(amount)
  }

  pub fn withdraw(
    &self,
    storage: &mut dyn Storage,
  ) -> ContractResult<Uint128> {
    // decrement delegator counts
    if self.has_delegation(storage, DelegationType::Growth)? {
      decrement(storage, &GROWTH_DELEGATOR_COUNT, 1)?;
    }
    if self.has_delegation(storage, DelegationType::Profit)? {
      decrement(storage, &PROFIT_DELEGATOR_COUNT, 1)?;
    }

    increment(storage, &SNAPSHOT_SEQ_NO, Uint128::one())?;

    // compute the total amount delegated by the user
    let (x_deleg_growth, x_deleg_profit) = self.get_delegation_amounts(storage)?;
    let x_delegation = x_deleg_growth + x_deleg_profit;

    // compute user's total growth and loss in their share of the pool's overall liquidity
    let (x_growth, x_loss) = self.claim(storage, DelegationType::Growth, false)?;

    // compute any unclaimed profit hanging around for the user
    let mut profit_delta =
      self.claim(storage, DelegationType::Profit, false)?.0 + self.memoized_profit;

    // compute amount to subtract from global liquidity amount
    let mut liquidity_delta =
      (x_delegation + x_growth + self.memoized_growth) - (x_loss + self.memoized_loss);

    let mut balance =
      (x_delegation + x_growth + self.memoized_growth) - (x_loss + self.memoized_loss);

    NET_PROFIT.update(storage, |net_profit| -> ContractResult<_> {
      profit_delta = profit_delta.min(net_profit);
      Ok(net_profit - profit_delta)
    })?;

    balance += profit_delta;

    NET_LIQUIDITY.update(storage, |net_liquidity| -> ContractResult<_> {
      if liquidity_delta > net_liquidity {
        let overflow_amount = liquidity_delta - net_liquidity;
        liquidity_delta -= overflow_amount;
        balance -= overflow_amount;
      }
      Ok(net_liquidity - liquidity_delta)
    })?;

    decrement(storage, &NET_GROWTH_DELEGATION, x_deleg_growth)?;
    decrement(storage, &NET_PROFIT_DELEGATION, x_deleg_profit)?;

    // remove Delegations
    self.remove_delegations(storage, DelegationType::Growth);
    self.remove_delegations(storage, DelegationType::Profit);

    Ok(balance)
  }

  fn remove_delegations(
    &self,
    storage: &mut dyn Storage,
    delegation_type: DelegationType,
  ) {
    let (delegations_map, seq_no_map) = match delegation_type {
      DelegationType::Growth => (&GROWTH_DELEGATIONS, &GROWTH_DELEGATIONS_SEQ_NO),
      DelegationType::Profit => (&PROFIT_DELEGATIONS, &PROFIT_DELEGATIONS_SEQ_NO),
    };

    let indices: Vec<u128> = delegations_map
      .prefix(self.owner.clone())
      .range(storage, None, None, Order::Ascending)
      .map(|r| r.unwrap().0)
      .collect();

    seq_no_map.remove(storage, self.owner.clone());

    for i in indices.iter() {
      delegations_map.remove(storage, (self.owner.clone(), *i));
    }
  }

  pub fn get_delegation_amounts(
    &self,
    storage: &dyn Storage,
  ) -> ContractResult<(Uint128, Uint128)> {
    Ok((
      if let Some((_, deleg)) = self.get_latest_delegation(storage, DelegationType::Growth)? {
        deleg.amount
      } else {
        Uint128::zero()
      },
      if let Some((_, deleg)) = self.get_latest_delegation(storage, DelegationType::Profit)? {
        deleg.amount
      } else {
        Uint128::zero()
      },
    ))
  }

  pub fn claim(
    &self,
    storage: &mut dyn Storage,
    target: DelegationType,
    is_amortizing: bool,
  ) -> ContractResult<(Uint128, Uint128)> {
    let delegations_map = match target {
      DelegationType::Growth => &GROWTH_DELEGATIONS,
      DelegationType::Profit => &PROFIT_DELEGATIONS,
    };

    let delegations = self.load_delegations(storage, &delegations_map)?;

    if delegations.is_empty() {
      return Ok((Uint128::zero(), Uint128::zero()));
    }

    let mut total_growth = Uint128::zero();
    let mut total_loss = Uint128::zero();

    if delegations.len() > 1 {
      for i in 0..delegations.len() - 1 {
        let (d0_index, d0) = &delegations[i];
        let d1 = &delegations[i + 1].1;
        if d0.i_snapshot < d1.i_snapshot {
          let (growth, loss) = self.process_delegation(storage, target.clone(), d0, Some(&d1))?;
          total_growth += growth;
          total_loss += loss;
        }
        delegations_map.remove(storage, (self.owner.clone(), *d0_index));
      }
    }

    // we only process the final delegation record if this claim call isn't
    // happening as a result of calling amortize
    if !is_amortizing {
      if let Some((d0_index, d0)) = delegations.last() {
        let (growth, loss) = self.process_delegation(storage, target.clone(), d0, None)?;
        total_growth += growth;
        total_loss += loss;
        let i_next_snapshot = Snapshot::get_next_index(storage)?;
        delegations_map.update(
          storage,
          (self.owner.clone(), *d0_index),
          |maybe_d0| -> ContractResult<_> {
            let mut d0 = maybe_d0.unwrap();
            d0.i_snapshot = Uint128::from(i_next_snapshot);
            Ok(d0)
          },
        )?;
      }
    }

    Ok((total_growth, total_loss))
  }

  fn load_delegations(
    &self,
    storage: &dyn Storage,
    map: &Map<(Addr, u128), Delegation>,
  ) -> ContractResult<Vec<(u128, Delegation)>> {
    Ok(
      map
        .prefix(self.owner.clone())
        .range(storage, None, None, Order::Ascending)
        .map(|result| result.unwrap())
        .collect(),
    )
  }

  fn process_delegation(
    &self,
    storage: &mut dyn Storage,
    target: DelegationType,
    d0: &Delegation,
    maybe_d1: Option<&Delegation>,
  ) -> ContractResult<(Uint128, Uint128)> {
    let d1_snapshot_index = if let Some(d1) = maybe_d1 {
      d1.i_snapshot.u128()
    } else {
      SNAPSHOTS_INDEX.load(storage)?.u128() + 1
    };

    let mut stale_snapshot_indices: Vec<u128> = vec![];
    let mut updated_snapshots: Vec<(u128, Snapshot)> = vec![];

    let amounts = match target {
      DelegationType::Growth => {
        let mut total_growth = Uint128::zero();
        let mut total_loss = Uint128::zero();
        for i_snapshot in d0.i_snapshot.u128()..d1_snapshot_index {
          if let Some(mut s) = SNAPSHOTS.may_load(storage, i_snapshot)? {
            let x_total = s.get_total_delegation();
            total_growth += s.growth.multiply_ratio(d0.amount, x_total);
            total_loss += s.loss.multiply_ratio(d0.amount, s.growth_delegation);

            s.claims_remaining -= 1;
            if s.claims_remaining == 0 {
              stale_snapshot_indices.push(i_snapshot);
            } else {
              updated_snapshots.push((i_snapshot, s));
            }
          }
        }
        (total_growth, total_loss)
      },
      DelegationType::Profit => {
        let mut total_growth = Uint128::zero();
        for i_snapshot in d0.i_snapshot.u128()..d1_snapshot_index {
          if let Some(mut s) = SNAPSHOTS.may_load(storage, i_snapshot)? {
            total_growth += s.growth.multiply_ratio(d0.amount, s.get_total_delegation());

            s.claims_remaining -= 1;
            if s.claims_remaining == 0 {
              stale_snapshot_indices.push(i_snapshot);
            } else {
              updated_snapshots.push((i_snapshot, s))
            }
          }
        }
        (total_growth, Uint128::zero())
      },
    };

    for i in stale_snapshot_indices.iter() {
      SNAPSHOTS.remove(storage, *i);
    }

    decrement(storage, &SNAPSHOTS_LEN, stale_snapshot_indices.len() as u32)?;

    for (i, s) in updated_snapshots.iter() {
      SNAPSHOTS.save(storage, *i, s)?;
    }

    Ok(amounts)
  }

  pub fn claim_readonly(
    &self,
    storage: &dyn Storage,
    target: DelegationType,
  ) -> ContractResult<(Uint128, Uint128)> {
    let delegations_map = match target {
      DelegationType::Growth => &GROWTH_DELEGATIONS,
      DelegationType::Profit => &PROFIT_DELEGATIONS,
    };

    let delegations = self.load_delegations(storage, &delegations_map)?;

    if delegations.is_empty() {
      return Ok((Uint128::zero(), Uint128::zero()));
    }

    let mut total_growth = Uint128::zero();
    let mut total_loss = Uint128::zero();

    if delegations.len() > 1 {
      for i in 0..delegations.len() - 1 {
        let (_, d0) = &delegations[i];
        let d1 = &delegations[i + 1].1;
        if d0.i_snapshot < d1.i_snapshot {
          let (growth, loss) =
            self.process_delegation_readonly(storage, target.clone(), d0, Some(&d1))?;
          total_growth += growth;
          total_loss += loss;
        }
      }
    }

    if let Some((_, d0)) = delegations.last() {
      let (growth, loss) = self.process_delegation_readonly(storage, target.clone(), d0, None)?;
      total_growth += growth;
      total_loss += loss;
    }

    Ok((total_growth, total_loss))
  }

  fn process_delegation_readonly(
    &self,
    storage: &dyn Storage,
    target: DelegationType,
    d0: &Delegation,
    maybe_d1: Option<&Delegation>,
  ) -> ContractResult<(Uint128, Uint128)> {
    let d1_snapshot_index = if let Some(d1) = maybe_d1 {
      d1.i_snapshot.u128()
    } else {
      SNAPSHOTS_INDEX.load(storage)?.u128() + 1
    };

    let amounts = match target {
      DelegationType::Growth => {
        let mut total_growth = Uint128::zero();
        let mut total_loss = Uint128::zero();
        for i_snapshot in d0.i_snapshot.u128()..d1_snapshot_index {
          if let Some(s) = SNAPSHOTS.may_load(storage, i_snapshot)? {
            let x_total = s.get_total_delegation();
            total_growth += s.growth.multiply_ratio(d0.amount, x_total);
            total_loss += s.loss.multiply_ratio(d0.amount, s.growth_delegation);
          }
        }
        (total_growth, total_loss)
      },
      DelegationType::Profit => {
        let mut total_growth = Uint128::zero();
        for i_snapshot in d0.i_snapshot.u128()..d1_snapshot_index {
          if let Some(s) = SNAPSHOTS.may_load(storage, i_snapshot)? {
            total_growth += s.growth.multiply_ratio(d0.amount, s.get_total_delegation());
          }
        }
        (total_growth, Uint128::zero())
      },
    };

    Ok(amounts)
  }

  pub fn memoize_claim_amounts(
    &mut self,
    storage: &mut dyn Storage,
  ) -> ContractResult<()> {
    let (growth, loss) = self.claim(storage, DelegationType::Growth, true)?;
    let profit = self.claim(storage, DelegationType::Profit, true)?.0;

    self.memoized_growth += growth;
    self.memoized_loss += loss;
    self.memoized_profit += profit;

    DELEGATION_ACCOUNTS.save(storage, self.owner.clone(), self)?;
    Ok(())
  }
}

impl Snapshot {
  pub fn get_latest(storage: &mut dyn Storage) -> ContractResult<Option<(u128, Self)>> {
    if SNAPSHOTS_LEN.load(storage)? == 0 {
      return Ok(None);
    }
    let idx = SNAPSHOTS_INDEX.load(storage)?.u128() - 1;
    Ok(Some((idx, SNAPSHOTS.load(storage, idx)?)))
  }

  pub fn get_next_index(storage: &dyn Storage) -> ContractResult<u128> {
    Ok(SNAPSHOTS_INDEX.load(storage)?.u128())
  }
  pub fn get_count(storage: &mut dyn Storage) -> ContractResult<u32> {
    Ok(SNAPSHOTS_LEN.load(storage)?)
  }

  pub fn get_next_index_and_increment(storage: &mut dyn Storage) -> ContractResult<u128> {
    Ok(
      SNAPSHOTS_INDEX
        .update(storage, |x| -> ContractResult<_> { Ok(x + Uint128::one()) })?
        .u128()
        - 1,
    )
  }

  pub fn get_total_delegation(&self) -> Uint128 {
    self.growth_delegation + self.profit_delegation
  }

  pub fn upsert(
    storage: &mut dyn Storage,
    growth: Uint128,
    loss: Uint128,
  ) -> ContractResult<Self> {
    let seq_no = SNAPSHOT_SEQ_NO.load(storage)?;

    // try to update the latest existing snapshot
    if let Some((i_prev_snapshot, mut prev_snapshot)) = Self::get_latest(storage)? {
      if prev_snapshot.seq_no == seq_no {
        prev_snapshot.growth += growth;
        prev_snapshot.loss += loss;
        SNAPSHOTS.save(storage, i_prev_snapshot, &prev_snapshot)?;
        return Ok(prev_snapshot);
      }
    }

    let i_snapshot = Self::get_next_index_and_increment(storage)?;
    let growth_delegation = NET_GROWTH_DELEGATION.load(storage)?;
    let profit_delegation = NET_PROFIT_DELEGATION.load(storage)?;
    let claims_remaining =
      GROWTH_DELEGATOR_COUNT.load(storage)? + PROFIT_DELEGATOR_COUNT.load(storage)?;

    // if we didn't just end up updating the previous snapshot,
    // we create and return a new one...
    let snapshot = Snapshot {
      seq_no,
      claims_remaining,
      growth_delegation,
      profit_delegation,
      growth,
      loss,
    };

    SNAPSHOTS.save(storage, i_snapshot, &snapshot)?;

    increment(storage, &SNAPSHOTS_LEN, 1)?;

    Ok(snapshot)
  }
}
