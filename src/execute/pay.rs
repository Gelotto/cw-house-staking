use crate::{
  models::{Account, ContractResult, Snapshot},
  state::{GLTO_CW20_CONTRACT_ADDR, NET_LIQUIDITY},
  util::decrement,
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_cw20_transfer_msg;

pub fn pay(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
  recipient: Addr,
  amount: Uint128,
) -> ContractResult<Response> {
  if amount.is_zero() {
    return Err(crate::error::ContractError::MissingAmount {});
  }

  Snapshot::upsert(deps.storage, deps.api, Uint128::zero(), amount)?;
  Account::amortize_claim_function(deps.storage, deps.api, 5)?;

  decrement(deps.storage, &NET_LIQUIDITY, amount)?;

  Ok(
    Response::new()
      .add_attributes(vec![
        attr("action", "pay"),
        attr("amount", amount.to_string()),
        attr("recipient", recipient.to_string()),
      ])
      .add_submessage(build_cw20_transfer_msg(
        &info.sender,
        &Addr::unchecked(GLTO_CW20_CONTRACT_ADDR),
        amount,
      )?),
  )
}
