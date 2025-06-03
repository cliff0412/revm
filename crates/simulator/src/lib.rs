pub mod fork_db;
pub mod utils;
use fork_db::fork_factory::ForkFactory;

use alloy_network::{AnyNetwork, Ethereum, Network};
use alloy_provider::{DynProvider, Provider, ProviderBuilder, WsConnect};
use std::sync::{Arc, Mutex, RwLock};

pub struct EvmSimulator {
    pub ws_provider: DynProvider<Ethereum>,
    pub fork_factory: Option<Arc<Mutex<ForkFactory>>>,
}

pub struct EvmFork {
    pub fork_factory: Arc<RwLock<ForkFactory>>,
}

impl EvmFork {
    pub fn new(fork_factory: Arc<RwLock<ForkFactory>>) -> Self {
        Self { fork_factory }
    }
    
}

impl EvmSimulator {
    pub async fn new(ws_url: &str) -> Self {
        let ws = WsConnect::new(ws_url);
        let provider = ProviderBuilder::new().connect_ws(ws).await.unwrap();

        EvmSimulator {
            ws_provider: provider.erased(),
            fork_factory: None,
        }
    }

}
