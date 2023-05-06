use cosmwasm_std::{to_binary, Addr, Coin, StdResult, Uint128, WasmMsg};

use crate::msg::ExecuteMsg;

pub struct House {
  pub address: Addr,
}

impl House {
  pub fn new(addr: &Addr) -> Self {
    Self {
      address: addr.clone(),
    }
  }

  pub fn build_send_payment_msg(
    &self,
    recipient: &Addr,
    amount: Uint128,
  ) -> StdResult<WasmMsg> {
    Ok(WasmMsg::Execute {
      contract_addr: self.address.clone().into(),
      msg: to_binary(&ExecuteMsg::SendPayment {
        recipient: recipient.clone(),
        amount,
      })?,
      funds: vec![],
    })
  }

  pub fn build_receive_payment_msg(
    &self,
    sender: Option<Addr>,
    amount: Uint128,
    funds: &Vec<Coin>,
  ) -> StdResult<WasmMsg> {
    Ok(WasmMsg::Execute {
      contract_addr: self.address.clone().into(),
      msg: to_binary(&ExecuteMsg::ReceivePayment { sender, amount })?,
      funds: funds.clone(),
    })
  }
}
