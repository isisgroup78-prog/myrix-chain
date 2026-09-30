//! Deterministic EVM execution boundary for MYRIX.
//!
//! This module is deliberately isolated from the account/ledger model until
//! the chain defines canonical EVM account, code and storage persistence.

use revm::{
    primitives::{Address, Bytes, ExecutionResult, TransactTo, U256},
    Evm,
};
use serde::{Deserialize, Serialize};

pub const DEFAULT_BLOCK_GAS_LIMIT: u64 = 30_000_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvmCall {
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub data: String,
    pub gas_limit: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvmExecution {
    pub succeeded: bool,
    pub gas_used: u64,
    pub output: String,
}

pub struct EvmExecutor {
    pub block_gas_limit: u64,
}

impl Default for EvmExecutor {
    fn default() -> Self {
        Self::new(DEFAULT_BLOCK_GAS_LIMIT)
    }
}

impl EvmExecutor {
    pub fn new(block_gas_limit: u64) -> Self {
        Self { block_gas_limit }
    }

    pub fn validate_call(&self, call: &EvmCall) -> Result<(Address, Option<Address>, U256, Bytes), String> {
        if call.gas_limit == 0 || call.gas_limit > self.block_gas_limit {
            return Err("invalid EVM gas limit".to_string());
        }

        let from = parse_address(&call.from)?;
        let to = call.to.as_deref().map(parse_address).transpose()?;
        let value = parse_u256_hex(&call.value)?;
        let data = parse_bytes(&call.data)?;
        Ok((from, to, value, data))
    }

    /// Executes a call against an empty in-memory EVM state.
    ///
    /// This is a deterministic execution primitive for tests and simulation.
    /// It is NOT yet the consensus state transition used by MYRIX blocks.
    pub fn simulate(&self, call: &EvmCall) -> Result<EvmExecution, String> {
        let (from, to, value, data) = self.validate_call(call)?;

        let mut evm = Evm::default();
        let tx = evm.tx_mut();
        tx.caller = from;
        tx.transact_to = match to {
            Some(address) => TransactTo::Call(address),
            None => TransactTo::Create,
        };
        tx.value = value;
        tx.data = data;
        tx.gas_limit = call.gas_limit;

        let result = evm.transact().map_err(|err| format!("EVM execution failed: {err:?}"))?;
        let gas_used = result.result.gas_used();
        let (succeeded, output) = match result.result {
            ExecutionResult::Success { output, .. } => (true, hex::encode(output.into_data())),
            ExecutionResult::Revert { output, .. } => (false, hex::encode(output)),
            ExecutionResult::Halt { .. } => (false, String::new()),
        };

        Ok(EvmExecution { succeeded, gas_used, output: format!("0x{output}") })
    }
}

fn parse_address(value: &str) -> Result<Address, String> {
    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.len() != 40 {
        return Err("EVM address must contain 20 bytes".to_string());
    }
    let bytes = hex::decode(raw).map_err(|_| "invalid EVM address".to_string())?;
    Address::try_from(bytes.as_slice()).map_err(|_| "invalid EVM address".to_string())
}

fn parse_bytes(value: &str) -> Result<Bytes, String> {
    let raw = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(raw).map_err(|_| "invalid EVM calldata".to_string())?;
    Ok(Bytes::from(bytes))
}

fn parse_u256_hex(value: &str) -> Result<U256, String> {
    let raw = value.strip_prefix("0x").unwrap_or(value);
    if raw.is_empty() {
        return Ok(U256::ZERO);
    }
    U256::from_str_radix(raw, 16).map_err(|_| "invalid EVM value".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_address() {
        let executor = EvmExecutor::default();
        let call = EvmCall {
            from: "0x01".into(),
            to: None,
            value: "0x0".into(),
            data: "0x".into(),
            gas_limit: 21_000,
        };
        assert!(executor.validate_call(&call).is_err());
    }

    #[test]
    fn rejects_zero_gas() {
        let executor = EvmExecutor::default();
        let call = EvmCall {
            from: "0x0000000000000000000000000000000000000001".into(),
            to: None,
            value: "0x0".into(),
            data: "0x".into(),
            gas_limit: 0,
        };
        assert!(executor.validate_call(&call).is_err());
    }
}
