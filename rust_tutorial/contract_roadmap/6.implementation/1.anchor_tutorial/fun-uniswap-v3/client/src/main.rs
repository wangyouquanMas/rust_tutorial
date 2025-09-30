use anchor_client::{Client, Cluster};
use anyhow::{format_err, Result};
use solana_client::{
    rpc_client::RpcClient,
};
use clap::Parser; 
use fun_uniswap_v3::{
    libraries::{tick_math},
    states::{POOL_SEED,POOL_TICK_ARRAY_BITMAP_SEED},
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair,Signer},
    system_program,
    transaction::Transaction,
    program_pack::Pack,
};
use configparser::ini::Ini;
use std::rc::Rc;
use spl_token_2022::{
    state::Mint,
};
use std::str::FromStr;

mod instructions;
use instructions::utils::*;
use instructions::amm_instructions::*;
use instructions::rpc::*;

#[derive(Debug, Parser)]
pub enum CommandsName {
    CreatePool {
        #[arg(long)]
        config_index: u16,
        #[arg(long)]
        price: f64,
        #[arg(long)]
        mint0: Pubkey,
        #[arg(long)]
        mint1: Pubkey,
        #[arg(short, long, default_value_t = 0)]
        open_time: u64,
    },
    CreateConfig {
        config_index: u16,
        tick_spacing: u16,
        trade_fee_rate: u32,
        protocol_fee_rate: u32,
        fund_fee_rate: u32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientConfig {
    http_url: String,
    ws_url: String,
    payer_path: String,
    admin_path: String,
    raydium_v3_program: Pubkey,
    slippage: f64,
    amm_config_key: Pubkey,

    mint0: Option<Pubkey>,
    mint1: Option<Pubkey>,
    pool_id_account: Option<Pubkey>,
    tickarray_bitmap_extension: Option<Pubkey>,
    amm_config_index: u16,
}

#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(subcommand)]
    pub command: CommandsName,
}

fn load_cfg(client_config: &String) -> Result<ClientConfig> {
    let mut config = Ini::new();
    let _map = config.load(client_config).unwrap();
    let http_url = config.get("Global", "http_url").unwrap();
    if http_url.is_empty() {
        panic!("http_url must not be empty");
    }
    let ws_url = config.get("Global", "ws_url").unwrap();
    if ws_url.is_empty() {
        panic!("ws_url must not be empty");
    }
    let payer_path = config.get("Global", "payer_path").unwrap();
    if payer_path.is_empty() {
        panic!("payer_path must not be empty");
    }
    let admin_path = config.get("Global", "admin_path").unwrap();
    if admin_path.is_empty() {
        panic!("admin_path must not be empty");
    }

    let raydium_v3_program_str = config.get("Global", "raydium_v3_program").unwrap();
    if raydium_v3_program_str.is_empty() {
        panic!("raydium_v3_program must not be empty");
    }
    let raydium_v3_program = Pubkey::from_str(&raydium_v3_program_str).unwrap();
    let slippage = config.getfloat("Global", "slippage").unwrap().unwrap();

    let mut mint0 = None;
    let mint0_str = config.get("Pool", "mint0").unwrap();
    if !mint0_str.is_empty() {
        mint0 = Some(Pubkey::from_str(&mint0_str).unwrap());
    }
    let mut mint1 = None;
    let mint1_str = config.get("Pool", "mint1").unwrap();
    if !mint1_str.is_empty() {
        mint1 = Some(Pubkey::from_str(&mint1_str).unwrap());
    }
    let amm_config_index = config.getuint("Pool", "amm_config_index").unwrap().unwrap() as u16;

    let (amm_config_key, __bump) = Pubkey::find_program_address(
        &[
            fun_uniswap_v3::states::AMM_CONFIG_SEED.as_bytes(),
            &amm_config_index.to_be_bytes(),
        ],
        &raydium_v3_program,
    );

    let pool_id_account = if mint0 != None && mint1 != None {
        if mint0.unwrap() > mint1.unwrap() {
            let temp_mint = mint0;
            mint0 = mint1;
            mint1 = temp_mint;
        }
        Some(
            Pubkey::find_program_address(
                &[
                    fun_uniswap_v3::states::POOL_SEED.as_bytes(),
                    amm_config_key.to_bytes().as_ref(),
                    mint0.unwrap().to_bytes().as_ref(),
                    mint1.unwrap().to_bytes().as_ref(),
                ],
                &raydium_v3_program,
            )
            .0,
        )
    } else {
        None
    };
    let tickarray_bitmap_extension = if pool_id_account != None {
        Some(
            Pubkey::find_program_address(
                &[
                    POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                    pool_id_account.unwrap().to_bytes().as_ref(),
                ],
                &raydium_v3_program,
            )
            .0,
        )
    } else {
        None
    };
    println!("tickarray_bitmap_extension: {:?}", tickarray_bitmap_extension);

    Ok(ClientConfig {
        http_url,
        ws_url,
        payer_path,
        admin_path,
        raydium_v3_program,
        slippage,
        amm_config_key,
        mint0,
        mint1,
        pool_id_account,
        tickarray_bitmap_extension,
        amm_config_index,
    })
}
fn read_keypair_file(s: &str) -> Result<Keypair> {
    solana_sdk::signature::read_keypair_file(s)
        .map_err(|_| format_err!("failed to read keypair from {}", s))
}

fn main() -> Result<()> {
    println!("Starting...");
    let client_config = "client_config.ini";
    let pool_config = load_cfg(&client_config.to_string()).unwrap();
    // Admin and cluster params.
    let payer = read_keypair_file(&pool_config.payer_path)?;
    let admin = read_keypair_file(&pool_config.admin_path)?;
    // solana rpc client
    let rpc_client = RpcClient::new(pool_config.http_url.to_string());

    // anchor client.
    let anchor_config = pool_config.clone();
    let url = Cluster::Custom(anchor_config.http_url, anchor_config.ws_url);
    let wallet = read_keypair_file(&pool_config.payer_path)?;
    let anchor_client = Client::new(url, Rc::new(wallet));
    let program = anchor_client.program(pool_config.raydium_v3_program)?;

    let opts = Opts::parse();
    match opts.command{
        CommandsName::CreateConfig {
            config_index,
            tick_spacing,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
        } => {
            let create_instr = create_amm_config_instr(
                &pool_config.clone(),
                config_index,
                tick_spacing,
                trade_fee_rate,
                protocol_fee_rate,
                fund_fee_rate,
            )?;
            // send
            let signers = vec![&payer, &admin];
            let recent_hash = rpc_client.get_latest_blockhash()?;
            let txn = Transaction::new_signed_with_payer(
                &create_instr,
                Some(&payer.pubkey()),
                &signers,
                recent_hash,
            );
            let signature = send_txn(&rpc_client, &txn, true)?;
            println!("{}", signature);
        }
        CommandsName::CreatePool {
            config_index,
            price,
            mint0,
            mint1,
            open_time,
        } => {
            let mut price = price;
            let mut mint0 = mint0;
            let mut mint1 = mint1;
            
            // DEBUG: Print original inputs
            println!("DEBUG CreatePool command:");
            println!("  Original mint0: {}", mint0);
            println!("  Original mint1: {}", mint1);
            println!("  Original price: {}", price);
            
            if mint0 > mint1 {
                std::mem::swap(&mut mint0, &mut mint1);
                price = 1.0 / price;
                println!("  Tokens swapped!");
            }
            println!("  Final mint0: {}", mint0);
            println!("  Final mint1: {}", mint1);
            println!("  Final price: {}", price);
            
            let load_pubkeys = vec![mint0, mint1];
            let rsps = rpc_client.get_multiple_accounts(&load_pubkeys)?;
            let mint0_owner = rsps[0].clone().unwrap().owner;
            let mint1_owner = rsps[1].clone().unwrap().owner;
            
            // Handle native SOL token (mint0) differently
            let mint0_decimals = if mint0_owner == system_program::id() {
                9 // SOL has 9 decimals
            } else {
                let mint0_account = spl_token::state::Mint::unpack(&rsps[0].as_ref().unwrap().data).unwrap();
                mint0_account.decimals
            };
            
            let mint1_account = spl_token::state::Mint::unpack(&rsps[1].as_ref().unwrap().data).unwrap();
            let mint1_decimals = mint1_account.decimals;
            
            let sqrt_price_x64 = price_to_sqrt_price_x64(price, mint0_decimals, mint1_decimals);
            let (amm_config_key, __bump) = Pubkey::find_program_address(
                &[
                    fun_uniswap_v3::states::AMM_CONFIG_SEED.as_bytes(),
                    &config_index.to_be_bytes(),
                ],
                &pool_config.raydium_v3_program,
            );
            let tick = tick_math::get_tick_at_sqrt_price(sqrt_price_x64).unwrap();
            println!(
                "tick:{}, price:{}, sqrt_price_x64:{}, amm_config_key:{}",
                tick, price, sqrt_price_x64, amm_config_key
            );

            // Calculate the correct bitmap based on the final ordered mints
            let (pool_key, _) = Pubkey::find_program_address(
                &[
                    fun_uniswap_v3::states::POOL_SEED.as_bytes(),
                    amm_config_key.to_bytes().as_ref(),
                    mint0.to_bytes().as_ref(),
                    mint1.to_bytes().as_ref(),
                ],
                &pool_config.raydium_v3_program,
            );
            
            let (correct_bitmap, _) = Pubkey::find_program_address(
                &[
                    POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                    pool_key.to_bytes().as_ref(),
                ],
                &pool_config.raydium_v3_program,
            );
            
            println!("DEBUG final calculations:");
            println!("  Using correct bitmap: {}", correct_bitmap);

            let create_pool_instr = create_pool_instr(
                &pool_config.clone(),
                amm_config_key,
                mint0,
                mint1,
                mint0_owner,
                mint1_owner,
                correct_bitmap, // Use the correctly calculated bitmap
                sqrt_price_x64,
                open_time,
            )?;

            // send
            let signers = vec![&payer];
            let recent_hash = rpc_client.get_latest_blockhash()?;
            let txn = Transaction::new_signed_with_payer(
                &create_pool_instr,
                Some(&payer.pubkey()),
                &signers,
                recent_hash,
            );
            let signature = send_txn(&rpc_client, &txn, true)?;
            println!("{}", signature);
        }
    }
    Ok(())
}