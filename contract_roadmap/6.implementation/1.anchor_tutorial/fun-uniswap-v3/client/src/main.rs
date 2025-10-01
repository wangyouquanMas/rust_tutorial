use anchor_client::{Client, Cluster};
use anyhow::{format_err, Result};
use solana_client::{
    rpc_client::RpcClient,
};
use clap::{Parser, Subcommand}; 
use fun_uniswap_v3::{
    states::{AMM_CONFIG_SEED},
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
use instructions::amm_instructions::*;
use instructions::rpc::*;
use instructions::utils::*;

#[derive(Debug, Subcommand)]
pub enum CommandsName {
    CreateConfig {
        #[arg(long)]
        config_index: u16,
        #[arg(long)]
        tick_spacing: u16,
        #[arg(long)]
        trade_fee_rate: u32,
        #[arg(long)]
        protocol_fee_rate: u32,
        #[arg(long)]
        fund_fee_rate: u32,
    },
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
        amm_config_index,
    })
}

fn read_keypair_file(s: &str) -> Result<Keypair> {
    solana_sdk::signature::read_keypair_file(s)
        .map_err(|_| format_err!("failed to read keypair from {}", s))
}

fn main() -> Result<()> {
    println!("Starting...");
    let opts = Opts::parse();

    let client_config = match std::env::var("CLIENT_CONFIG") {
        Ok(path) if !path.is_empty() => path,
        _ => "client_config.ini".to_string(),
    };
    let pool_config = load_cfg(&client_config)?;
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

           //TODO: custom process to make mint0 < mint1
           // price = mint1 / mint0
           if mint0 > mint1 {
             std::mem::swap(&mut mint0, &mut mint1);
             price = 1.0 / price;
             println!("Token swapped!")
           }
            
           let load_pubkeys = vec![mint0,mint1];
           let rsps = rpc_client.get_multiple_accounts(&load_pubkeys)?;

           let mint0_owner = rsps[0].clone().unwrap().owner;
           let mint1_owner = rsps[1].clone().unwrap().owner;

        //handle native SQL token (mint0) differently
        //TODO: system program is the owner of wsol 
        let mint0_decimals = if mint0_owner == system_program::id(){
            9 // SOL has 9 decimals 
        }else{
            let mint0_account = spl_token::state::Mint::unpack(&rsps[0].as_ref().unwrap().data).unwrap();
            mint0_account.decimals
        };
        //TODO: Token mint data structure contains the field: decimals
        let mint1_account = spl_token::state::Mint::unpack(&rsps[1].as_ref().unwrap().data).unwrap();
        let mint1_decimals = mint1_account.decimals;

        //TODO: Formula to do conversion.
        let sqrt_price_x64  = price_to_sqrt_price_x64(price,mint0_decimals,mint1_decimals);

        println!("mint0:{}, mint1:{}, price:{}, sqrt_price_x64:{}", mint0_decimals, mint1_decimals, price, sqrt_price_x64);

        //get amm config key
        let (amm_config_key, __bump) = Pubkey::find_program_address(
            &[
                fun_uniswap_v3::states::AMM_CONFIG_SEED.as_bytes(),
                &config_index.to_be_bytes(),
            ],
            &pool_config.raydium_v3_program,
        );

        println!("amm_config_key:{}",amm_config_key)


        }
    }
    Ok(())
}