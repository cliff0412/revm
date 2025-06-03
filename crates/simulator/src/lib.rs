pub mod fork_db;

use fork_db::fork_factory::ForkFactory;


use alloy_network::{AnyNetwork, Ethereum, Network};
use alloy_provider::{Provider, DynProvider};
use std::sync::{Arc, Mutex};

pub struct EvmSimulator {
    pub ws_provider: DynProvider<Ethereum>,
    // pub weth_address: Address,
    // pub target_pools: Option<Arc<DashMap<Address, DefiStorage>>>,
    // pub latest_block_info: Arc<RwLock<BlockInfo>>,
    pub fork_factory: Option<Arc<Mutex<ForkFactory>>>,
}