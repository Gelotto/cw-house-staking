use crate::{
  models::{ContractResult, DelegationAccount},
  state::{DELEGATION_ACCOUNTS, DELEGATION_ACCOUNTS_LEN, NET_LIQUIDITY, NET_PROFIT, TOKEN},
  util::decrement,
};
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::utils::funds::build_send_submsg;

pub fn withdraw(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  // total number oxisting delegation acounts:
  let n_accounts = DelegationAccount::get_count(deps.storage)?;

  // process the DelegationAccount's outstanding delegation, claiming whatever liquidity
  // and profit is owed.
  let amount =
    if let Some(account) = DELEGATION_ACCOUNTS.may_load(deps.storage, info.sender.clone())? {
      let mut amount = account.withdraw(deps.storage)?;

      // adjust contract-level profit and liquidity accumulators:
      if n_accounts == Uint128::one() {
        NET_PROFIT.update(deps.storage, |dust| -> ContractResult<_> {
          amount += dust;
          Ok(Uint128::zero())
        })?;

        NET_LIQUIDITY.update(deps.storage, |dust| -> ContractResult<_> {
          amount += dust;
          Ok(Uint128::zero())
        })?;
      }

      // remove the account
      DELEGATION_ACCOUNTS.remove(deps.storage, info.sender.clone());

      // adjust DelegationAccount counter
      decrement(deps.storage, &DELEGATION_ACCOUNTS_LEN, Uint128::one())?;

      amount
    } else {
      Uint128::zero()
    };

  // build response with token transfer submsg
  let mut resp = Response::new().add_attributes(vec![
    attr("action", "withdraw"),
    attr("amount", amount.to_string()),
  ]);

  if !amount.is_zero() {
    resp = resp.add_submessage(build_send_submsg(
      &info.sender,
      amount,
      &TOKEN.load(deps.storage)?,
    )?);
  }

  Ok(resp)
}
