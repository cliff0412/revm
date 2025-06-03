pub mod fork_db;

use fork_db::fork_factory::ForkFactory;

use alloy_network::{AnyNetwork, Ethereum, Network};
use alloy_provider::{DynProvider, Provider, ProviderBuilder, WsConnect};
use std::sync::{Arc, Mutex, RwLock};

pub struct EvmSimulator {
    pub ws_provider: DynProvider<Ethereum>,
    // pub weth_address: Address,
    // pub target_pools: Option<Arc<DashMap<Address, DefiStorage>>>,
    // pub latest_block_info: Arc<RwLock<BlockInfo>>,
    pub fork_factory: Option<Arc<Mutex<ForkFactory>>>,
}

// #[derive(Debug)]
// pub enum ReplayTransactionResult {
//     Success {
//         gas_used: u64,
//         gas_refunded: u64,
//         output: StdBytes,
//     },
//     Revert {
//         gas_used: u64,
//         message: String,
//     },
// }

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

    // pub async fn fork_at(
    //     &self,
    //     block_number: u64,
    // ) -> Result<EvmFork, SimulationError<Provider<Ws>>> {
    //     match self.fork_factory {
    //         Some(ref f) => {
    //             let g = f.lock().await;
    //             if g.fork_block.as_u64() != block_number {
    //                 return Err(SimulationError::BlockNumberUnmatch(
    //                     g.fork_block.as_u64(),
    //                     block_number,
    //                 ));
    //             }
    //             let evm_fork = EvmFork::new(Arc::clone(f));
    //             Ok(evm_fork)
    //         }
    //         None => Err(SimulationError::ForkfactoryNotReady),
    //     }
    // }

    // pub async fn replay_transaction(
    //     &self,
    //     tx_hash: TxHash,
    // ) -> Result<ReplayTransactionResult, SimulationError<Provider<Ws>>> {
    //     let tx = self
    //         .ws_provider
    //         .get_transaction(tx_hash)
    //         .await?
    //         .ok_or(SimulationError::TransactionNotFound(tx_hash))?;

    //     if tx.block_number.is_none() {
    //         return Err(SimulationError::TransactionBlkNumberNotFound);
    //     }

    //     let tx_block_number = tx.block_number.unwrap().as_u64();

    //     let cache_db = CacheDB::new(EmptyDB::default());
    //     let fork_factory = ForkFactory::new_sandbox_factory(
    //         self.ws_provider.clone(),
    //         cache_db,
    //         (tx_block_number - 1).into(),
    //     );

    //     let mut evm = revm::EVM::new();
    //     let fork_db = fork_factory.new_sandbox_fork();
    //     evm.database(fork_db);

    //     evm.env.block.number = rU256::from(tx_block_number);
    //     let block = self.ws_provider.get_block_with_txs(tx_block_number).await?;

    //     if let Some(ref block) = block {
    //         evm.env.block.timestamp = block.timestamp.into();
    //         evm.env.block.coinbase = block.author.unwrap_or_default().into();
    //         evm.env.block.difficulty = block.difficulty.into();
    //         evm.env.block.prevrandao = block.mix_hash.map(h256_to_b256);
    //         evm.env.block.basefee = block.base_fee_per_gas.unwrap_or_default().into();
    //         evm.env.block.gas_limit = block.gas_limit.into();
    //     }

    //     // execure front txs
    //     if let Some(block) = block {
    //         for (_, tx) in block.transactions.into_iter().enumerate() {
    //             // arbitrum L1 transaction at the start of every block that has gas price 0
    //             // and gas limit 0 which causes reverts, so we skip it
    //             if tx.from == ARBITRUM_SENDER {
    //                 continue;
    //             }
    //             if tx.hash().eq(&tx_hash) {
    //                 break;
    //             }

    //             configure_tx_env(&mut evm.env, &tx);
    //             let inspector = NoOpInspector {};
    //             let _run_result = match evm.inspect_commit(inspector) {
    //                 Ok(result) => result,
    //                 Err(e) => {
    //                     eprintln!("simulate error for other tx {:?},{:?}", tx.hash, e);
    //                     return Err(SimulationError::SimulationEvmOtherTxError(format!(
    //                         "{:?},{:?}",
    //                         tx.hash, e
    //                     )));
    //                 }
    //             };
    //         }
    //     }

    //     // execure target tx
    //     configure_tx_env(&mut evm.env, &tx);
    //     if let Some(_to) = tx.to {
    //         // println!("executing call transaction");
    //         let inspector = NoOpInspector {};
    //         let run_result = match evm.inspect_commit(inspector) {
    //             Ok(result) => result,
    //             Err(e) => {
    //                 eprintln!("simulate error for target tx {:?},{:?}", tx.hash, e);
    //                 return Err(SimulationError::SimulationEvmError(format!(
    //                     "{:?},{:?}",
    //                     tx.hash, e
    //                 )));
    //             }
    //         };
    //         // println!("result: {:?}", run_result);
    //         let replay_ret = match run_result {
    //             ExecutionResult::Success {
    //                 gas_used,
    //                 gas_refunded,
    //                 output,
    //                 ..
    //             } => match output {
    //                 Output::Call(o) => ReplayTransactionResult::Success {
    //                     gas_used,
    //                     gas_refunded,
    //                     output: o.clone(),
    //                 },
    //                 Output::Create(_o, _) => unimplemented!(),
    //             },
    //             ExecutionResult::Revert { gas_used, output } => {
    //                 // println!("reverted with output: {:?}", output);
    //                 let ret = decode_revert(&output, None, None);
    //                 match ret {
    //                     Ok(r) => ReplayTransactionResult::Revert {
    //                         gas_used,
    //                         message: r,
    //                     },
    //                     Err(e) => {
    //                         eprintln!("error: {:?}", e);
    //                         return Err(SimulationError::DecodeRevertMsgError);
    //                     }
    //                 }
    //             }
    //             ExecutionResult::Halt {
    //                 reason: _,
    //                 gas_used: _,
    //             } => unimplemented!(),
    //         };
    //         return Ok(replay_ret);
    //     }
    //     Err(SimulationError::UnableToReplay(tx_hash))
    // }
}
