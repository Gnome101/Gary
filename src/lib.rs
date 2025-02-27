#![allow(dead_code)]

use blueprint_sdk::alloy::sol;
use blueprint_sdk::macros::load_abi;
use serde::{Deserialize, Serialize};
use std::net::AddrParseError;
use thiserror::Error;
use blueprint_sdk::alloy::primitives::{Address, Bytes};

pub mod constants;
pub mod contexts;
pub mod jobs;
#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Job error: {0}")]
    Job(String),
    #[error("Chain error: {0}")]
    Chain(String),
    #[error("Context error: {0}")]
    Context(String),
    #[error("Event conversion error: {0}")]
    Conversion(String),
    #[error("Parse error: {0}")]
    Parse(#[from] AddrParseError),
    #[error("Event Listener Processor error: {0}")]
    Processor(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

type ProcessorError =
    blueprint_sdk::event_listeners::core::Error<blueprint_sdk::event_listeners::evm::error::Error>;

impl From<Error>
    for blueprint_sdk::event_listeners::core::Error<
        blueprint_sdk::event_listeners::evm::error::Error,
    >
{
    fn from(value: Error) -> Self {
        blueprint_sdk::event_listeners::core::Error::ProcessorError(
            blueprint_sdk::event_listeners::evm::error::Error::Client(value.to_string()),
        )
    }
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Serialize, Deserialize)]
    IncredibleSquaringTaskManager,
    "contracts/out/IncredibleSquaringTaskManager.sol/IncredibleSquaringTaskManager.json"
);

load_abi!(
    INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
    "contracts/out/IncredibleSquaringTaskManager.sol/IncredibleSquaringTaskManager.json"
);

// Add semicolons to struct declarations
sol!(
    #[sol(event)]
    #[derive(Debug)]
    pub struct EncryptedValueSubmitted {
        #[sol(indexed)]
        pub sender: Address,
        pub c1: Bytes,
        pub c2: Bytes,
    }
);

sol!(
    #[sol(event)]
    #[derive(Debug)]
    pub struct DecryptionRequested {
        #[sol(indexed)]
        pub requester: Address,
    }
);

// Import the missing BN254 and IBLSSignatureChecker types
sol!(
    #[allow(missing_docs)]
    #[derive(Debug)]
    BN254,
    "eigenlayer-middleware/src/libraries/BN254.sol:BN254"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    IBLSSignatureChecker,
    "eigenlayer-middleware/src/BLSSignatureChecker.sol:BLSSignatureChecker"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    PauserRegistry,
    "./contracts/out/IPauserRegistry.sol/IPauserRegistry.json"
);