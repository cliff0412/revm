use alloy_network::Ethereum;
use alloy_provider::{DynProvider, Empty, Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types_eth::BlockId;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::{anyhow, Result};
use context::{Context, TxEnv};
use context_interface::result::{ExecutionResult, Output};
use database::{AlloyDB, CacheDB, EmptyDB, EmptyDBTyped};
use database_interface::WrapDatabaseAsync;
use handler::{ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext};
use primitives::{address, keccak256, Address, Bytes, StorageKey, TxKind, KECCAK_EMPTY, U256};
use revm_simulator::utils::crypto::gen_random_address;
use state::{AccountInfo, Bytecode};
use std::convert::Infallible;
use std::sync::Arc;

type AlloyCacheDB = CacheDB<WrapDatabaseAsync<AlloyDB<Ethereum, DynProvider>>>;

pub const USDC_TOKEN: Address = address!("0x74b7f16337b8972027f6196a17a631ac6de26d22");

pub fn usdc_token_holder_balance_storage(holder: Address) -> StorageKey {
    let slot: StorageKey = keccak256((holder, U256::from(9)).abi_encode()).into();
    slot
}

fn erc20_balance_of(token: Address, address: Address, alloy_db: &mut AlloyCacheDB) -> Result<U256> {
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
    cache_db: &mut AlloyCacheDB,
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
    let one_ether = U256::from(1_000_000_000_000_000_000u128);
    let one_usdc = U256::from(1_000_000u128);

    let rpc_url = "https://rpc.xlayer.tech/unlimited/abc";
    let provider = ProviderBuilder::new().connect(rpc_url).await?.erased();
    // let ws = WsConnect::new(rpc_url);
    // let provider = ProviderBuilder::new().connect_ws(ws).await?;

    let usdc_code = provider.get_code_at(USDC_TOKEN).await.unwrap();
    let usdc_byte_code = Bytecode::new_raw(usdc_code);
    let usdc_code_hash = usdc_byte_code.hash_slow();
    let usdc_token_acc_info = AccountInfo {
        nonce: 0_u64,
        balance: U256::from(0),
        code_hash: usdc_code_hash,
        code: Some(usdc_byte_code),
    };

    let alloy_db = WrapDatabaseAsync::new(AlloyDB::new(provider, BlockId::latest())).unwrap();
    let mut cache_db = CacheDB::new(alloy_db);
    cache_db.insert_account_info(USDC_TOKEN, usdc_token_acc_info);

    const N_ACCTS: usize = 100;
    let mut accts: [Address; N_ACCTS] = [Address::ZERO; N_ACCTS];
    for i in 0..N_ACCTS {
        accts[i] = gen_random_address();
        // fund each sending acct with 1ether
        let acc_info = AccountInfo {
            nonce: 0_u64,
            balance: one_ether,
            code_hash: KECCAK_EMPTY,
            code: None,
        };
        cache_db.insert_account_info(accts[i], acc_info);
        // fund each sending acct with 1000usdc
        let _ = cache_db
            .insert_account_storage(
                USDC_TOKEN,
                usdc_token_holder_balance_storage(accts[i]),
                one_usdc * U256::from(1000),
            )
            .unwrap();
    }

    let acc_usdc_balance_before = erc20_balance_of(USDC_TOKEN, accts[0], &mut cache_db)?;
    assert_eq!(acc_usdc_balance_before, U256::from(1000_000_000));

    let account_to = address!("0x6c4cde76cacaf67b00d471e4c5fa45c0c2a526f6");
    let acc_to_info = AccountInfo {
        nonce: 0_u64,
        balance: one_ether,
        code_hash: KECCAK_EMPTY,
        code: None,
    };
    cache_db.insert_account_info(account_to, acc_to_info);
    let _ = cache_db
        .insert_account_storage(
            USDC_TOKEN,
            usdc_token_holder_balance_storage(account_to),
            one_usdc * U256::from(0),
        )
        .unwrap();

    let acc_usdc_balance_before = erc20_balance_of(USDC_TOKEN, account_to, &mut cache_db)?;
    assert_eq!(acc_usdc_balance_before, U256::from(0));

    let mut nonce = 0;

    // WARM UP
    println!("start WARM UP!");
    for i in 0..5 {
        println!("warmup step: {:?}", i);
        for j in 0..N_ACCTS {
            erc20_transfer(
                accts[j],
                nonce,
                account_to,
                U256::from(100),
                USDC_TOKEN,
                &mut cache_db,
            )?;
        }

        nonce = nonce + 1;
    }
    println!("finish WARM UP!");

    let start = std::time::Instant::now();
    const N_ITERS: usize = 1000;
    for i in 0..N_ITERS {
        for j in 0..N_ACCTS {
            erc20_transfer(
                accts[j],
                nonce,
                account_to,
                U256::from(100),
                USDC_TOKEN,
                &mut cache_db,
            )?;
        }

        nonce = nonce + 1;
    }
    println!(
        "total time spent for {:?} erc20 transfers: {:?} ms, TPS: {:?}",
        N_ITERS * N_ACCTS,
        start.elapsed().as_millis(),
        N_ITERS * N_ACCTS * 1000 / start.elapsed().as_millis() as usize
    );

    let acc_usdc_balance_after = erc20_balance_of(USDC_TOKEN, account_to, &mut cache_db)?;
    assert_eq!(acc_usdc_balance_after, U256::from(10050000));
    Ok(())
}
