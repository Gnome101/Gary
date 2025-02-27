use blueprint_sdk::alloy::network::EthereumWallet;
use blueprint_sdk::alloy::primitives::Address;
use blueprint_sdk::alloy::signers::local::PrivateKeySigner;
use blueprint_sdk::logging::info;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::eigenlayer::bls::EigenlayerBLSConfig;
use blueprint_sdk::utils::evm::get_wallet_provider_http;

use gary::constants::{AGGREGATOR_PRIVATE_KEY, TASK_MANAGER_ADDRESS};
use gary::contexts::aggregator::AggregatorContext;
use gary::contexts::client::AggregatorClient;
use gary::contexts::x_square::EigenSquareContext;
use gary::jobs::compute_x_square::XsquareEigenEventHandler;
use gary::jobs::initialize_task::InitializeBlsTaskEventHandler;
use gary::jobs::elgamal_aggregator::{
    handle_encrypted_value_submitted, handle_decryption_requested
};

use gary::IncredibleSquaringTaskManager;

// --- ElGamal imports ---
use elastic_elgamal::{Keypair, group::Ristretto, DiscreteLogTable};
use rand::thread_rng;

// For sleeping in a loop
use std::time::Duration;
use tokio::time::sleep;
use tracing::error;
#[blueprint_sdk::main(env)]
async fn main() -> color_eyre::Result<()> {
    // 1) Parse aggregator's EVM signer
    let signer: PrivateKeySigner = AGGREGATOR_PRIVATE_KEY
        .parse()
        .expect("failed to generate wallet ");
    let wallet = EthereumWallet::from(signer);

    // 2) Provider for EVM transactions
    let provider = get_wallet_provider_http(&env.http_rpc_endpoint, wallet.clone());

    // 3) Build aggregator + eigen context
    let server_address = format!("{}:{}", "127.0.0.1", 8081);
    let eigen_client_context = EigenSquareContext {
        client: AggregatorClient::new(&server_address)?,
        std_config: env.clone(),
    };
    let aggregator_context = AggregatorContext::new(
        server_address,
        *TASK_MANAGER_ADDRESS,
        wallet,
        env.clone()
    )
    .await
    .unwrap();

    // 4) Instantiate your contract wrapper
    let contract = IncredibleSquaringTaskManager::IncredibleSquaringTaskManagerInstance::new(
        *TASK_MANAGER_ADDRESS,
        provider,
    );

    // 5) Build your event handlers
    let initialize_task =
        InitializeBlsTaskEventHandler::new(contract.clone(), aggregator_context.clone());
    let x_square_eigen = XsquareEigenEventHandler::new(contract.clone(), eigen_client_context);

  let encrypted_value_handler = handle_encrypted_value_submitted::new(... aggregator_context.clone() ...);
    let decryption_requested_handler = handle_decryption_requested::new(... aggregator_context.clone() ...);

    // 6) (New) Generate aggregator’s ElGamal keypair
    let mut rng = thread_rng();
    let (pk, sk) = Keypair::<Ristretto>::generate(&mut rng).into_tuple();
    info!("~~~ ElGamal Key Generation (Aggregator) ~~~");
    info!("Public key (pk) = {:?}", pk);
    // For security reasons, do NOT log your `sk` in a real system:
    // info!("Secret key (sk) = {:?}", sk);

    // (Optional) Here you might want to store `pk` on-chain (via a contract call),
    // or simply log it so that users can encrypt against it. For example:
    // contract.storePublicKey(pk_bytes, ...).send().await?;

    // 7) For demonstration, if you want aggregator itself to post some example ciphertexts:
    // In a real scenario, users do this from separate wallets / code.
    // This loop is just to show how you might do repeated encryption & submission.
//    tokio::spawn(async move {
//     // Build a lookup table for small plaintext sums [0..200].
//     let lookup_table = DiscreteLogTable::new(0..200);

//     let mut aggregated_cipher: Option<elastic_elgamal::Ciphertext<Ristretto>> = None;

//     // Suppose we ingest 5 ciphertexts
//     for i in 1..=5 {
//         let plaintext_value = i as u64;
//         let enc_i = pk.encrypt(plaintext_value, &mut thread_rng());
//         info!("Encrypting value {} -> ciphertext: {:?}", i, enc_i);

//         // Move old aggregator ciphertext out (if any), add new one
//         aggregated_cipher = Some(match aggregated_cipher.take() {
//             None => enc_i,
//             Some(prev) => prev + enc_i,  // `+` consumes both
//         });

//         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//     }

//     // Now decrypt the final sum
//     if let Some(final_cipher) = aggregated_cipher {
//         if let Some(plaintext_sum) = sk.decrypt(final_cipher, &lookup_table) {
//             info!("Decrypted final sum of user inputs: {}", plaintext_sum);
//         } else {
//             error!("Could not decrypt sum (maybe out of the lookup table range).");
//         }
//     }
// });


    // 8) Run your normal blueprint jobs
    info!("~~~ Executing the incredible squaring blueprint ~~~");
    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default());
    BlueprintRunner::new(eigen_config, env)
        .job(initialize_task)
        .job(x_square_eigen)
        .job(encrypted_value_handler)
        .job(decryption_requested_handler)
        // aggregator_context can run in background if needed
        .background_service(Box::new(aggregator_context))
        .run()
        .await?;

    info!("Exiting...");
    Ok(())
}
