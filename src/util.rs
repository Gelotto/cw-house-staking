use cosmwasm_std::{Addr, Api, Storage, Uint128};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::{error::ContractError, models::ContractResult};

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

pub fn decrement<T>(
  storage: &mut dyn Storage,
  item: &Item<T>,
  increment: T,
) -> ContractResult<T>
where
  T: DeserializeOwned + Serialize + std::ops::Sub<Output = T>,
{
  item.update(storage, |x| -> ContractResult<_> { Ok(x - increment) })
}

pub fn mul_pct(
  total: Uint128,
  pct: Uint128,
) -> Uint128 {
  total.multiply_ratio(pct, Uint128::from(1000u128))
}

pub fn validate_addr(
  api: &dyn Api,
  addr_str: &String,
) -> ContractResult<Addr> {
  api
    .addr_validate(addr_str)
    .map_err(|_| ContractError::InvalidAddress {})
}
