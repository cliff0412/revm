use alloy_provider::{Provider, ProviderBuilder, WsConnect};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a WebSocket endpoint
    let rpc_url = "wss://eth-mainnet.g.alchemy.com/v2/4K8OralYxsI-NM8-i9kkf";
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().connect_ws(ws).await?;

    // Example: Get chain ID
    let chain_id = provider.get_chain_id().await?;
    println!("Chain ID: {}", chain_id);

    Ok(())
}
