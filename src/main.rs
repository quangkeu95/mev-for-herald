use std::sync::Arc;

use artemis_core::{
    collectors::mevshare_collector::MevShareCollector,
    engine::Engine,
    executors::mev_share_executor::MevshareExecutor,
    types::{CollectorMap, ExecutorMap},
};
use clap::Parser;
use mev_for_herald::simple_arbitrage::{self, SimpleArbitrageStrategy};
use tracing::{info, Level};
use tracing_subscriber::{filter, prelude::*};

use ethers::{
    prelude::MiddlewareBuilder,
    providers::{Provider, Ws},
    signers::{LocalWallet, Signer},
};

/// CLI Options.
#[derive(Parser, Debug)]
pub struct Args {
    /// MEV Share SSE url endpoint
    #[arg(long, env)]
    pub mev_share_sse_url: String,
    /// Ethereum node WS endpoint.
    #[arg(long, env)]
    pub wss: String,
    /// Private key for sending txs.
    #[arg(long, env)]
    pub private_key: String,
    /// MEV share signer
    #[arg(long, env)]
    pub flashbots_signer: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up tracing and parse args.
    let filter = filter::Targets::new()
        .with_target("mev_for_herald", Level::INFO)
        .with_target("artemis_core", Level::INFO);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let args = Args::parse();

    info!(?args, "Running MEV for herald...");

    //  Set up providers and signers.
    let ws = Ws::connect(args.wss).await?;
    let provider = Provider::new(ws);

    let wallet: LocalWallet = args.private_key.parse().unwrap();
    let address = wallet.address();

    let provider = Arc::new(provider.nonce_manager(address).with_signer(wallet.clone()));
    let fb_signer: LocalWallet = args.flashbots_signer.parse().unwrap();

    // Set up engine.
    let mut engine: Engine<simple_arbitrage::Event, simple_arbitrage::Action> = Engine::default();

    // Set up collector.
    let mevshare_collector = Box::new(MevShareCollector::new(args.mev_share_sse_url));
    let mevshare_collector =
        CollectorMap::new(mevshare_collector, simple_arbitrage::Event::MEVShareEvent);
    engine.add_collector(Box::new(mevshare_collector));

    // Set up strategy.
    let strategy = SimpleArbitrageStrategy::new();
    engine.add_strategy(Box::new(strategy));

    // Set up executor.
    let mev_share_executor = Box::new(MevshareExecutor::new(fb_signer));
    let mev_share_executor = ExecutorMap::new(mev_share_executor, |action| match action {
        simple_arbitrage::Action::SubmitBundle(bundle) => Some(bundle),
    });
    engine.add_executor(Box::new(mev_share_executor));

    // info!("starting engine...");

    // Start engine.
    if let Ok(mut set) = engine.run().await {
        while let Some(res) = set.join_next().await {
            info!("res: {:?}", res);
        }
    }

    Ok(())
}
