use blueprint_sdk::alloy::network::EthereumWallet;
use blueprint_sdk::alloy::primitives::Address;
use blueprint_sdk::alloy::signers::local::PrivateKeySigner;
use blueprint_sdk::logging::info;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::eigenlayer::bls::EigenlayerBLSConfig;
use blueprint_sdk::utils::evm::get_wallet_provider_http;
use gary::constants::{
    AGGREGATOR_PRIVATE_KEY, TASK_MANAGER_ADDRESS,
};
use gary::contexts::aggregator::AggregatorContext;
use gary::contexts::client::AggregatorClient;
use gary::contexts::x_square::EigenSquareContext;
use gary::jobs::compute_x_square::XsquareEigenEventHandler;
use gary::jobs::initialize_task::InitializeBlsTaskEventHandler;
use gary::IncredibleSquaringTaskManager;

#[blueprint_sdk::main(env)]
async fn main() {
    let signer: PrivateKeySigner = AGGREGATOR_PRIVATE_KEY
        .parse()
        .expect("failed to generate wallet ");
    let wallet = EthereumWallet::from(signer);
    let provider = get_wallet_provider_http(&env.http_rpc_endpoint, wallet.clone());

    let server_address = format!("{}:{}", "127.0.0.1", 8081);
    let eigen_client_context = EigenSquareContext {
        client: AggregatorClient::new(&server_address)?,
        std_config: env.clone(),
    };
    let aggregator_context =
        AggregatorContext::new(server_address, *TASK_MANAGER_ADDRESS, wallet, env.clone())
            .await
            .unwrap();

    let contract = IncredibleSquaringTaskManager::IncredibleSquaringTaskManagerInstance::new(
        *TASK_MANAGER_ADDRESS,
        provider,
    );

    let initialize_task =
        InitializeBlsTaskEventHandler::new(contract.clone(), aggregator_context.clone());

    let x_square_eigen = XsquareEigenEventHandler::new(contract.clone(), eigen_client_context);

    info!("~~~ Executing the incredible squaring blueprint ~~~");
    let eigen_config = EigenlayerBLSConfig::new(Address::default(), Address::default());
    BlueprintRunner::new(eigen_config, env)
        .job(x_square_eigen)
        .job(initialize_task)
        .background_service(Box::new(aggregator_context))
        .run()
        .await?;

    info!("Exiting...");
    Ok(())
}
