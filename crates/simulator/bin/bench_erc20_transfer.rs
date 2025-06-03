use std::convert::Infallible;

use alloy_provider::{Empty, Provider, ProviderBuilder, WsConnect};
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::{anyhow, Result};
use context::{Context, TxEnv};
use context_interface::result::{ExecutionResult, Output};
use database::{CacheDB, EmptyDB, EmptyDBTyped};
use handler::{ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext};
use primitives::{address, keccak256, Address, Bytes, StorageKey, TxKind, KECCAK_EMPTY, U256};
use state::{AccountInfo, Bytecode};

pub const WETH_TOKEN: Address = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");

pub fn weth_token_balance_of_storage(address: Address) -> U256 {
    keccak256((address, U256::from(3)).abi_encode()).into()
}

fn erc20_balance_of(
    token: Address,
    address: Address,
    alloy_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> Result<U256> {
    sol! {
        function balanceOf(address account) public returns (uint256);
    }

    let encoded = balanceOfCall { account: address }.abi_encode();
    let mut evm = Context::mainnet().with_db(alloy_db).build_mainnet();

    let result = evm
        .transact(TxEnv {
            // 0x1 because calling weth proxy from zero address fails
            caller: address!("0000000000000000000000000000000000000001"),
            kind: TxKind::Call(token),
            data: encoded.into(),
            value: U256::from(0),
            ..Default::default()
        })
        .unwrap();

    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        result => return Err(anyhow!("'balanceOf' execution failed: {result:?}")),
    };
    let balance = <U256>::abi_decode(&value)?;

    Ok(balance)
}

fn erc20_transfer(
    from: Address,
    nonce: u64,
    to: Address,
    amount: U256,
    token: Address,
    cache_db: &mut CacheDB<EmptyDBTyped<Infallible>>,
) -> Result<()> {
    sol! {
        function transfer(address to, uint amount) external returns (bool);
    }

    let encoded = transferCall { to, amount }.abi_encode();

    let mut evm = Context::mainnet().with_db(cache_db).build_mainnet();

    let tx = TxEnv {
        caller: from,
        kind: TxKind::Call(token),
        data: encoded.into(),
        value: U256::from(0),
        nonce: nonce,
        ..Default::default()
    };

    let ref_tx = evm.transact_commit(tx).unwrap();
    let success: bool = match ref_tx {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => <bool>::abi_decode(&value)?,
        result => return Err(anyhow!("'transfer' execution failed: {result:?}")),
    };

    if !success {
        return Err(anyhow!("'transfer' failed"));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut cache_db = CacheDB::new(EmptyDB::new());

    let rpc_url = "wss://eth-mainnet.g.alchemy.com/v2/4K8OralYxsI-NM8-i9kkf";
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().connect_ws(ws).await?;
    let weth_token_code = provider.get_code_at(WETH_TOKEN).await?;
    let weth_token_code = Bytecode::new_raw(weth_token_code);
    let weth_token_code_hash = weth_token_code.hash_slow();

    let acc_info_weth_token = AccountInfo {
        nonce: 0_u64,
        balance: U256::ZERO,
        code_hash: weth_token_code_hash,
        code: Some(weth_token_code),
    };
    cache_db.insert_account_info(WETH_TOKEN, acc_info_weth_token);

    let account = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");
    let account_to = address!("0x26b34A00B1E6D50D21cA793ee01eEe89189f38Ad");
    let one_ether = U256::from(1_000_000_000_000_000_000u128);
    let one_weth = U256::from(1_000_000u128);
    let hashed_acc_balance_slot = weth_token_balance_of_storage(account);
    cache_db
        .insert_account_storage(WETH_TOKEN, hashed_acc_balance_slot.into(), one_ether)
        .unwrap();

    let acc_info = AccountInfo {
        nonce: 0_u64,
        balance: one_ether,
        code_hash: KECCAK_EMPTY,
        code: None,
    };
    cache_db.insert_account_info(account, acc_info);
    let acc_weth_balance_before = erc20_balance_of(WETH_TOKEN, account, &mut cache_db)?;
    println!("weth balance before swap: {}", acc_weth_balance_before);

    let mut nonce = 0;
    let start = std::time::Instant::now();
    for i in 0..100000 {
        erc20_transfer(
            account,
            nonce,
            account_to,
            U256::from(100),
            WETH_TOKEN,
            &mut cache_db,
        )?;
        nonce = nonce + 1;
    }
    println!(
        "total time spent for 10k erc20 transfers: {:?}",
        start.elapsed().as_millis()
    );

    let acc_weth_balance_after = erc20_balance_of(WETH_TOKEN, account, &mut cache_db)?;
    println!(
        "weth balance of sending acct after transfer: {}",
        acc_weth_balance_before
    );

    let acc_weth_balance_after = erc20_balance_of(WETH_TOKEN, account_to, &mut cache_db)?;
    println!(
        "weth balance of receiving acct after transfer: {}",
        acc_weth_balance_after
    );
    Ok(())
}
