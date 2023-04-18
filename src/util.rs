use cosmwasm_std::{Api, Storage};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::models::ContractResult;

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

pub fn log(
  api: &dyn Api,
  tag: &str,
  msg: String,
) {
  let mut prefixed_msg = String::from(">>>");
  prefixed_msg.push_str(format!(" [{}] ", tag).to_uppercase().as_str());
  prefixed_msg.push_str(&msg);
  api.debug(&prefixed_msg);
}
