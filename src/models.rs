use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Order, Storage, Uint128};
use cw_storage_plus::Map;

use crate::{
  error::ContractError,
  state::{
    ACCOUNTS, GROWTH_DELEGATIONS, GROWTH_DELEGATIONS_SEQ_NO, GROWTH_DELEGATOR_COUNT,
    NET_GROWTH_DELEGATION, NET_LIQUIDITY, NET_PROFIT_DELEGATION, PROFIT_DELEGATIONS,
    PROFIT_DELEGATIONS_SEQ_NO, PROFIT_DELEGATOR_COUNT, SNAPSHOTS, SNAPSHOTS_LEN, SNAPSHOTS_SEQ_NO,
    SNAPSHOT_TICK,
  },
  util::{decrement, increment, log},
};

pub type ContractResult<T> = Result<T, ContractError>;

#[cw_serde]
pub enum DelegationType {
  Growth,
  Profit,
}

#[cw_serde]
pub struct Account {
  pub owner: Addr,
}

#[cw_serde]
pub struct Snapshot {
  pub tick: Uint128,
  pub claims_remaining: u32,
  pub growth_delegation: Uint128,
  pub profit_delegation: Uint128,
  pub income: Uint128,
  pub outlay: Uint128,
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

  pub fn get_next_index(storage: &mut dyn Storage) -> ContractResult<u128> {
    Ok(SNAPSHOTS_SEQ_NO.load(storage)?.u128())
  }
  pub fn get_count(storage: &mut dyn Storage) -> ContractResult<u128> {
    Ok(SNAPSHOTS_LEN.load(storage)?.u128())
  }

  pub fn get_next_index_and_increment(storage: &mut dyn Storage) -> ContractResult<u128> {
    Ok(
      SNAPSHOTS_SEQ_NO
        .update(storage, |x| -> ContractResult<_> { Ok(x + Uint128::one()) })?
        .u128()
        - 1,
    )
  }

  pub fn get_total_delegation(&self) -> Uint128 {
    self.growth_delegation + self.profit_delegation
  }

  pub fn create(
    storage: &mut dyn Storage,
    api: &dyn Api,
    income: Uint128,
    outlay: Uint128,
  ) -> ContractResult<Self> {
    let i_snapshot = Self::get_next_index_and_increment(storage)?;
    let growth_delegation = NET_GROWTH_DELEGATION.load(storage)?;
    let profit_delegation = NET_PROFIT_DELEGATION.load(storage)?;
    let claims_remaining =
      GROWTH_DELEGATOR_COUNT.load(storage)? + PROFIT_DELEGATOR_COUNT.load(storage)?;
    log(
      api,
      "snapshot::create",
      format!(
        "new snapshot index {} with {} claims remaining",
        i_snapshot, claims_remaining
      ),
    );
    let snapshot = Snapshot {
      tick: SNAPSHOT_TICK.load(storage)?,
      claims_remaining,
      growth_delegation,
      profit_delegation,
      income,
      outlay,
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

  pub fn stake(
    &self,
    storage: &mut dyn Storage,
    api: &dyn Api,
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
        log(
          api,
          "account::stake",
          format!(
            "updating existing Delegation with snapshot index: {}",
            i_next_snapshot
          ),
        );
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

    log(
      api,
      "account::stake",
      format!("new Delegation with snapshot index: {}", i_next_snapshot),
    );

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

  pub fn take_profit(
    &self,
    storage: &mut dyn Storage,
    api: &dyn Api,
  ) -> ContractResult<Uint128> {
    let balance = self.process(storage, api, DelegationType::Profit)?.0;
    increment(storage, &SNAPSHOT_TICK, Uint128::one())?;
    Ok(balance)
  }

  pub fn withdraw(
    &self,
    storage: &mut dyn Storage,
    api: &dyn Api,
  ) -> ContractResult<Uint128> {
    // decrement delegator counts
    if let Some(account) = ACCOUNTS.may_load(storage, self.owner.clone())? {
      if account.has_delegation(storage, DelegationType::Growth)? {
        decrement(storage, &GROWTH_DELEGATOR_COUNT, 1)?;
      }
      if account.has_delegation(storage, DelegationType::Profit)? {
        decrement(storage, &PROFIT_DELEGATOR_COUNT, 1)?;
      }
    } else {
      return Ok(Uint128::zero());
    }

    // compute the total amount delegated by the user
    let (x_deleg_growth, x_deleg_profit) = self.get_delegation_amounts(storage)?;
    let x_delegation = x_deleg_growth + x_deleg_profit;

    log(
      api,
      "account::withdraw",
      format!("withdraw processing growth delegations"),
    );

    // compute user's total gain and loss in their share of the pool's overall liquidity
    let (x_gain, x_loss) = self.process(storage, api, DelegationType::Growth)?;

    log(
      api,
      "account::withdraw",
      format!("processing profit delegations"),
    );

    // compute any unclaimed profit hanging around for the user
    let x_profit = self.process(storage, api, DelegationType::Profit)?.0;

    log(
      api,
      "account::withdraw",
      format!(
        "x_deleg_growth: {}, x_deleg_profit: {}, x_gain: {}, x_loss: {}",
        x_deleg_growth, x_deleg_profit, x_gain, x_loss
      ),
    );

    // compute amount to subtract from global liquidity amount
    let dx_liquidity = (x_delegation + x_gain) - x_loss;

    log(
      api,
      "account::withdraw",
      format!(
        "decrementing liqudity {} by {}",
        NET_LIQUIDITY.load(storage)?.u128(),
        dx_liquidity.u128()
      ),
    );

    NET_LIQUIDITY.update(storage, |x| -> ContractResult<_> {
      Ok(x - x.min(dx_liquidity))
    })?;

    log(
      api,
      "account::withdraw",
      format!(
        "decrementing net growth delegation {} by {}",
        NET_GROWTH_DELEGATION.load(storage)?.u128(),
        x_deleg_growth.u128()
      ),
    );

    decrement(storage, &NET_GROWTH_DELEGATION, x_deleg_growth)?;

    log(
      api,
      "account::withdraw",
      format!(
        "decrementing net profit delegation {} by {}",
        NET_PROFIT_DELEGATION.load(storage)?.u128(),
        x_deleg_profit.u128()
      ),
    );

    decrement(storage, &NET_PROFIT_DELEGATION, x_deleg_profit)?;

    // remove Delegations
    self.remove_delegations(storage, DelegationType::Growth);
    self.remove_delegations(storage, DelegationType::Profit);

    // balance is the total withdrawn amount
    let balance = (x_delegation + x_gain + x_profit) - x_loss;

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
    storage: &mut dyn Storage,
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

  fn process(
    &self,
    storage: &mut dyn Storage,
    api: &dyn Api,
    target: DelegationType,
  ) -> ContractResult<(Uint128, Uint128)> {
    let delegations_map = match target {
      DelegationType::Growth => &GROWTH_DELEGATIONS,
      DelegationType::Profit => &PROFIT_DELEGATIONS,
    };

    let delegations = self.load_delegations(storage, &delegations_map)?;

    log(
      api,
      "account::process",
      format!("processing {:?} {} delegations", target, delegations.len()),
    );

    log(
      api,
      "account::process",
      format!("snapshot count: {}", SNAPSHOTS_LEN.load(storage)?),
    );

    if delegations.is_empty() {
      return Ok((Uint128::zero(), Uint128::zero()));
    }

    let mut total_gain = Uint128::zero();
    let mut total_loss = Uint128::zero();

    if delegations.len() > 1 {
      for i in 0..delegations.len() - 1 {
        let (d0_index, d0) = &delegations[i];
        let d1 = &delegations[i + 1].1;
        if d0.i_snapshot < d1.i_snapshot {
          log(
            api,
            "account::process",
            format!(
              "delegation index {}, snapshot index: {}",
              d0_index, d0.i_snapshot
            ),
          );
          let (gain, loss) =
            self.process_delegation(storage, api, target.clone(), d0, Some(&d1))?;
          total_gain += gain;
          total_loss += loss;
        }
        delegations_map.remove(storage, (self.owner.clone(), *d0_index));
      }
    }

    if let Some((d0_index, d0)) = delegations.last() {
      log(
        api,
        "account::process",
        format!(
          "final delegation index {}, snapshot index: {}",
          d0_index, d0.i_snapshot
        ),
      );
      let (gain, loss) = self.process_delegation(storage, api, target.clone(), d0, None)?;
      total_gain += gain;
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

    Ok((total_gain, total_loss))
  }

  fn load_delegations(
    &self,
    storage: &mut dyn Storage,
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
    api: &dyn Api,
    target: DelegationType,
    d0: &Delegation,
    maybe_d1: Option<&Delegation>,
  ) -> ContractResult<(Uint128, Uint128)> {
    let d1_snapshot_index = if let Some(d1) = maybe_d1 {
      d1.i_snapshot.u128()
    } else {
      SNAPSHOTS_SEQ_NO.load(storage)?.u128() + 1
    };

    let mut stale_snapshot_indices: Vec<u128> = vec![];
    let mut updated_snapshots: Vec<(u128, Snapshot)> = vec![];

    let amounts = match target {
      DelegationType::Growth => {
        let mut total_gain = Uint128::zero();
        let mut total_loss = Uint128::zero();
        for i_snapshot in d0.i_snapshot.u128()..d1_snapshot_index {
          if let Some(mut s) = SNAPSHOTS.may_load(storage, i_snapshot)? {
            log(
              api,
              "account::process_delegation",
              format!("processing snapshot {} for growth delegation", i_snapshot),
            );

            let x_total = s.get_total_delegation();
            total_gain += s.income.multiply_ratio(d0.amount, x_total);
            total_loss += s.outlay.multiply_ratio(d0.amount, s.growth_delegation);

            s.claims_remaining -= 1;
            if s.claims_remaining == 0 {
              stale_snapshot_indices.push(i_snapshot);
            } else {
              updated_snapshots.push((i_snapshot, s));
            }
          }
        }
        (total_gain, total_loss)
      },
      DelegationType::Profit => {
        let mut total_gain = Uint128::zero();
        for i_snapshot in d0.i_snapshot.u128()..d1_snapshot_index {
          if let Some(mut s) = SNAPSHOTS.may_load(storage, i_snapshot)? {
            log(
              api,
              "account::process_delegation",
              format!("processing snapshot {} for profit delegation", i_snapshot),
            );

            total_gain += s
              .income
              .multiply_ratio(d0.amount, s.profit_delegation + s.growth_delegation);

            s.claims_remaining -= 1;
            if s.claims_remaining == 0 {
              stale_snapshot_indices.push(i_snapshot);
            } else {
              updated_snapshots.push((i_snapshot, s))
            }
          }
        }
        (total_gain, Uint128::zero())
      },
    };

    for i in stale_snapshot_indices.iter() {
      SNAPSHOTS.remove(storage, *i);
    }

    decrement(
      storage,
      &SNAPSHOTS_LEN,
      Uint128::from(stale_snapshot_indices.len() as u128),
    )?;

    for (i, s) in updated_snapshots.iter() {
      SNAPSHOTS.save(storage, *i, s)?;
    }

    Ok(amounts)
  }
}
