use blueprint_sdk::{
    event_listeners::evm::EvmContractEventListener,
    macros::job,
    logging::{info, error},
    alloy::primitives::{Bytes, Address},
};
use color_eyre::Result;
use std::convert::Infallible;

use crate::{
    AggregatorContext, 
    IncredibleSquaringTaskManager, 
    INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
    // your event definitions from lib.rs
    IncredibleSquaringTaskManager::EncryptedValueSubmitted,
    IncredibleSquaringTaskManager::DecryptionRequested,
};
use elastic_elgamal::Ciphertext;

/// Handler #1: On EncryptedValueSubmitted, we parse (c1,c2) and add to aggregator sum
#[job(
    id = 100,
    event_listener(
        listener = EvmContractEventListener<AggregatorContext, IncredibleSquaringTaskManager::EncryptedValueSubmitted>,
        instance = IncredibleSquaringTaskManager,
        abi = INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
        pre_processor = convert_encrypted_submission
    ),
)]
pub async fn handle_encrypted_value_submitted(
    ctx: AggregatorContext,
    c1: Bytes,
    c2: Bytes,
) -> std::result::Result<u32, Infallible> {
    info!("Received EncryptedValueSubmitted event");

    // Convert c1/c2 from 32-byte compressed Ristretto points into `Ciphertext<Ristretto>`.
    // We'll assume you wrote a helper to decompress them. In many codebases, you'll rely on curve25519-dalek:
    let ciphertext = match bytes_to_ciphertext(&c1, &c2) {
        Ok(ciph) => ciph,
        Err(e) => {
            error!("Failed to parse ciphertext: {}", e);
            return Ok(0);
        }
    };

    // Add to aggregator's running sum
    {
        let mut state = ctx.elgamal_state.lock().await;
        state.add_ciphertext(ciphertext);
    }
    info!("Successfully added ciphertext to aggregator sum");

    Ok(1)
}

/// Handler #2: On DecryptionRequested, aggregator finalizes & calls setDecryptionResult
#[job(
    id = 101,
    event_listener(
        listener = EvmContractEventListener<AggregatorContext, IncredibleSquaringTaskManager::DecryptionRequested>,
        instance = IncredibleSquaringTaskManager,
        abi = INCREDIBLE_SQUARING_TASK_MANAGER_ABI_STRING,
        pre_processor = convert_decryption_request
    ),
)]
pub async fn handle_decryption_requested(
    ctx: AggregatorContext,
) -> std::result::Result<u32, Infallible> {
    info!("Received DecryptionRequested event");

    // 1) Decrypt sum
    let sum_plaintext = {
        let mut state = ctx.elgamal_state.lock().await;
        state.decrypt_sum()
    };
    info!("Aggregator decrypted sum = {}", sum_plaintext);

    // 2) Call setDecryptionResult(...) on chain
    let contract = IncredibleSquaringTaskManager::IncredibleSquaringTaskManagerInstance::new(
        ctx.task_manager_address,
        ctx.wallet.clone(),
    );
    let tx_result = contract
        .setDecryptionResult(sum_plaintext)
        .from(ctx.wallet.address()) 
        .send()
        .await;

    match tx_result {
        Ok(call_res) => {
            info!("setDecryptionResult tx sent: hash = {:?}", call_res.tx_hash);
        }
        Err(e) => {
            error!("Failed to call setDecryptionResult: {}", e);
            return Ok(0);
        }
    }

    Ok(1)
}

/// Pre-processor for EncryptedValueSubmitted event
/// Returns `(c1, c2)` for the job function
pub async fn convert_encrypted_submission(
    (event, _log): (
        EncryptedValueSubmitted,
        blueprint_sdk::alloy::rpc::types::Log,
    ),
) -> Result<Option<(Bytes, Bytes)>, crate::Error> {
    let c1 = event.c1;
    let c2 = event.c2;
    Ok(Some((c1, c2)))
}

/// Pre-processor for DecryptionRequested event
pub async fn convert_decryption_request(
    (event, _log): (
        DecryptionRequested,
        blueprint_sdk::alloy::rpc::types::Log,
    ),
) -> Result<Option<()>, crate::Error> {
    // We do not need parameters for the job function
    Ok(Some(()))
}

/// Helper to parse c1/c2 (32 bytes each) into `Ciphertext<Ristretto>`
fn bytes_to_ciphertext(
    c1_bytes: &Bytes,
    c2_bytes: &Bytes,
) -> Result<Ciphertext<elastic_elgamal::group::Ristretto>, String> {
    if c1_bytes.len() != 32 || c2_bytes.len() != 32 {
        return Err("Invalid ciphertext length".into());
    }
    // Decompress each 32-byte compressed Ristretto
    // Typically you'd use curve25519-dalek to do:
    //   let c1_pt = CompressedRistretto(c1_bytes[..].try_into().unwrap()).decompress()...
    // For brevity, we show a hypothetical function:
    let c1_pt = decompress_ristretto_point(&c1_bytes[..])
        .ok_or("Failed to decompress c1")?;
    let c2_pt = decompress_ristretto_point(&c2_bytes[..])
        .ok_or("Failed to decompress c2")?;

    Ok(Ciphertext { c1: c1_pt, c2: c2_pt })
}

/// Example stand-in decompression method
fn decompress_ristretto_point(
    bytes: &[u8]
) -> Option<<elastic_elgamal::group::Ristretto as elastic_elgamal::group::Group>::Elem> {
    use curve25519_dalek::ristretto::CompressedRistretto;
    use curve25519_dalek::ristretto::RistrettoPoint;

    let comp = CompressedRistretto::from_slice(bytes);
    let decompressed: Option<RistrettoPoint> = comp.decompress();
    decompressed.map(|pt| pt)
}
