use crate::{
  models::{ContractResult, Snapshot},
  state::{NET_LIQUIDITY, SNAPSHOTS, SNAPSHOT_TICK},
  util::decrement,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};

pub fn pay(
  deps: DepsMut,
  _env: Env,
  _info: MessageInfo,
  recipient: Addr,
  amount: Uint128,
) -> ContractResult<Response> {
  let did_update: bool =
    if let Some((i_snapshot, mut snapshot)) = Snapshot::get_latest(deps.storage)? {
      if snapshot.tick == SNAPSHOT_TICK.load(deps.storage)? {
        snapshot.outlay += amount;
        SNAPSHOTS.save(deps.storage, i_snapshot, &snapshot)?;
        true
      } else {
        false
      }
    } else {
      false
    };

  if !did_update {
    Snapshot::create(deps.storage, deps.api, Uint128::zero(), amount)?;
  }

  decrement(deps.storage, &NET_LIQUIDITY, amount)?;

  Ok(Response::new().add_attributes(vec![
    attr("action", "pay"),
    attr("amount", amount.to_string()),
    attr("recipient", recipient.to_string()),
  ]))
}
