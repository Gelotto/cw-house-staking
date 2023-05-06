use crate::execute;
use crate::models::ContractResult;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query;
use crate::state;
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:house-staking-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> ContractResult<Response> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
  state::initialize(deps, &env, &info, &msg)?;
  Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> ContractResult<Response> {
  match msg {
    ExecuteMsg::Delegate { growth, profit } => execute::delegate(deps, env, info, growth, profit),
    ExecuteMsg::Withdraw {} => execute::withdraw(deps, env, info),
    ExecuteMsg::SendProfit {} => execute::send_profit(deps, env, info),
    ExecuteMsg::SetClient { address } => execute::set_client(deps, env, info, &address),
    ExecuteMsg::ReceivePayment { sender, amount } => {
      execute::receive_payment(deps, env, info, sender, amount)
    },
    ExecuteMsg::SendPayment { recipient, amount } => {
      execute::send_payment(deps, env, info, &recipient, amount)
    },
  }
}

#[entry_point]
pub fn query(
  deps: Deps,
  _env: Env,
  msg: QueryMsg,
) -> StdResult<Binary> {
  let result = match msg {
    QueryMsg::Select { fields, wallet } => to_binary(&query::select(deps, fields, wallet)?),
  }?;
  Ok(result)
}

#[entry_point]
pub fn migrate(
  _deps: DepsMut,
  _env: Env,
  _msg: MigrateMsg,
) -> ContractResult<Response> {
  Ok(Response::default())
}
