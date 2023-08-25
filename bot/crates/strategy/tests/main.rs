use anyhow::{anyhow, Result};
use std::{str::FromStr, sync::Arc};

use cfmms::pool::{Pool, UniswapV2Pool, UniswapV3Pool};
use ethers::{
    abi::{ParamType, Token},
    prelude::{abigen, Lazy},
    providers::{Middleware, Provider, Ws},
    types::{Address, Bytes, Transaction, TxHash, U64},
};
use strategy::{
    bot::SandoBot,
    types::{BlockInfo, RawIngredients, StratConfig},
};

// -- consts --
static WSS_RPC: &str = "ws://localhost:8545";
pub static WETH_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"
        .parse()
        .unwrap()
});

// -- utils --
fn setup_logger() {
    let _ = fern::Dispatch::new()
        .level(log::LevelFilter::Error)
        .level_for("strategy", log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply();
}

async fn setup_bot(provider: Arc<Provider<Ws>>) -> SandoBot<Provider<Ws>> {
    setup_logger();

    let strat_config = StratConfig {
        sando_address: "0x7AaCc5300ec7Ac58fe86645D08f21b1BEcadf99a"
            .parse()
            .unwrap(),
        sando_inception_block: U64::from(30895710),
        searcher_signer: "0x0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .unwrap(),
    };

    SandoBot::new(provider, strat_config)
}

async fn block_num_to_info(block_num: u64, provider: Arc<Provider<Ws>>) -> BlockInfo {
    let block = provider.get_block(block_num).await.unwrap().unwrap();

    block.try_into().unwrap()
}

fn hex_to_address(hex: &str) -> Address {
    hex.parse().unwrap()
}

async fn hex_to_univ2_pool(hex: &str, provider: Arc<Provider<Ws>>) -> Pool {
    let pair_address = hex_to_address(hex);
    let pool = UniswapV2Pool::new_from_address(pair_address, provider)
        .await
        .unwrap();
    Pool::UniswapV2(pool)
}

async fn hex_to_univ3_pool(hex: &str, provider: Arc<Provider<Ws>>) -> Pool {
    let pair_address = hex_to_address(hex);
    let pool = UniswapV3Pool::new_from_address(pair_address, provider)
        .await
        .unwrap();
    Pool::UniswapV3(pool)
}

async fn victim_tx_hash(tx: &str, provider: Arc<Provider<Ws>>) -> Transaction {
    println!("get tx");
    let tx_hash: TxHash = TxHash::from_str(tx).unwrap();
    provider.get_transaction(tx_hash).await.unwrap().unwrap()
}

/// testing against: https://eigenphi.io/mev/bsc/tx/0x920e4d80386a85ecc069f039298225242f4c3ebc93a5c67f5f18273c1596a7c6
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn can_sandwich_uni_v2() {
    let client = Arc::new(Provider::new(Ws::connect(WSS_RPC).await.unwrap()));

    let bot = setup_bot(client.clone()).await;

    let ingredients = RawIngredients::new(
        vec![
            victim_tx_hash(
                "0x9b1daff7fcdb24dd029f7250740a7762309d552f3bd006a8cd1a8237ec7a0607",
                client.clone(),
            )
            .await,
        ],
        *WETH_ADDRESS,
        hex_to_address("0xa68c9C2C39176b3Ee9F6359B68E853893C6dDc5a"),
        hex_to_univ2_pool("0x45fc0Bd45f7a3CE4a12EC46fa01a54A195a24645", client.clone()).await,
    );

    let target_block = block_num_to_info(30996860, client.clone()).await;
    let _ = bot
        .is_sandwichable(ingredients, target_block)
        .await
        .unwrap();
}

/// testing against: https://eigenphi.io/mev/ethereum/tx/0x056ede919e31be59b7e1e8676b3be1272ce2bbd3d18f42317a26a3d1f2951fc8
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn can_sandwich_sushi_swap() {
    let client = Arc::new(Provider::new(Ws::connect(WSS_RPC).await.unwrap()));

    let bot = setup_bot(client.clone()).await;

    let ingredients = RawIngredients::new(
        vec![
            victim_tx_hash(
                "0xb344fdc6a3b7c65c5dd971cb113567e2ee6d0636f261c3b8d624627b90694cdb",
                client.clone(),
            )
            .await,
        ],
        *WETH_ADDRESS,
        hex_to_address("0x3b484b82567a09e2588A13D54D032153f0c0aEe0"),
        hex_to_univ2_pool("0xB84C45174Bfc6b8F3EaeCBae11deE63114f5c1b2", client.clone()).await,
    );

    let target_block = block_num_to_info(16873148, client.clone()).await;

    let _ = bot
        .is_sandwichable(ingredients, target_block)
        .await
        .unwrap();
}

/// testing against: https://eigenphi.io/mev/ethereum/tx/0xc132e351e8c7d3d8763a894512bd8a33e4ca60f56c0516f7a6cafd3128bd59bb
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn can_sandwich_multi_v2_swaps() {
    let client = Arc::new(Provider::new(Ws::connect(WSS_RPC).await.unwrap()));

    let bot = setup_bot(client.clone()).await;

    let ingredients = RawIngredients::new(
        vec![
            victim_tx_hash(
                "0x4791d05bdd6765f036ff4ae44fc27099997417e3bdb053ecb52182bbfc7767c5",
                client.clone(),
            )
            .await,
            victim_tx_hash(
                "0x923c9ba97fea8d72e60c14d1cc360a8e7d99dd4b31274928d6a79704a8546eda",
                client.clone(),
            )
            .await,
        ],
        *WETH_ADDRESS,
        hex_to_address("0x31b16Ff7823096a227Aac78F1C094525A84ab64F"),
        hex_to_univ2_pool("0x657c6a08d49B4F0778f9cce1Dc49d196cFCe9d08", client.clone()).await,
    );

    let target_block = block_num_to_info(16780625, client.clone()).await;

    let _ = bot
        .is_sandwichable(ingredients, target_block)
        .await
        .unwrap();
}

/// testing against: https://eigenphi.io/mev/bsc/tx/0x37271dcdc2aae4e45a10d4c585797686b26b1469477b785209db50baa3aa45ba
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn can_sandwich_uni_v3() {
    let client = Arc::new(Provider::new(Ws::connect(WSS_RPC).await.unwrap()));

    let bot = setup_bot(client.clone()).await;

    let ingredients = RawIngredients::new(
        vec![
            victim_tx_hash(
                "0x6323b9f89e252015789b1d984d785883c385a3df11e0d694ee6281916ee307a9",
                client.clone(),
            )
            .await,
        ],
        *WETH_ADDRESS,
        hex_to_address("0xd98438889Ae7364c7E2A3540547Fad042FB24642"),
        hex_to_univ3_pool("0xA2C1e0237bF4B58bC9808A579715dF57522F41b2", client.clone()).await,
    );

    let target_block = block_num_to_info(31010233, client.clone()).await;

    let _ = bot
        .is_sandwichable(ingredients, target_block)
        .await
        .unwrap();
}
