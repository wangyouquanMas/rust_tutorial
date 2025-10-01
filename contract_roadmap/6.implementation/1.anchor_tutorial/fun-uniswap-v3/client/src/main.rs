use anchor_client::{Client, Cluster};
use anyhow::{format_err, Result};
use solana_client::{
    rpc_client::RpcClient,
    rpc_request::TokenAccountsFilter,
};
use clap::{Parser, Subcommand}; 
use fun_uniswap_v3::{
    libraries::{tick_math},
    states::{AMM_CONFIG_SEED, POOL_SEED, POOL_TICK_ARRAY_BITMAP_SEED},
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

use solana_account_decoder::{
    parse_token::{TokenAccountType, UiAccountState},
    UiAccountData, UiAccountEncoding,
};

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
    OpenPosition {
        #[arg(long)]
        tick_lower_price: f64,
        #[arg(long)]
        tick_upper_price: f64,
        #[arg(short, long)]
        is_base_0: bool,
        #[arg(long)]
        input_amount: u64,
        #[arg(short, long)]
        with_metadata: bool,
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

        println!("amm_config_key:{}",amm_config_key);

        let tick = tick_math::get_tick_at_sqrt_price(sqrt_price_x64).unwrap();
        println!(
            "tick:{}, price:{}, sqrt_price_x64:{}, amm_config_key:{}",
            tick, price, sqrt_price_x64, amm_config_key
        );

        // Calculate the correct bitmap based on the final ordered mints
        let (pool_key, _) = Pubkey::find_program_address(
                &[
                    POOL_SEED.as_bytes(),
                    amm_config_key.to_bytes().as_ref(),
                    mint0.to_bytes().as_ref(),
                    mint1.to_bytes().as_ref(),
                ],
                &pool_config.raydium_v3_program,
        );

        println!("pool_key:{}", pool_key);

        let (correct_bitmap, _) = Pubkey::find_program_address(
            &[
                POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                pool_key.to_bytes().as_ref(),
            ],
            &pool_config.raydium_v3_program,
        );

        println!("correct_bitmap:{}", correct_bitmap);



        let create_pool_instr = create_pool_instr(
                &pool_config.clone(),
                amm_config_key,
                mint0,
                mint1,
                mint0_owner,
                mint1_owner,
                correct_bitmap, // Use the correctly calculated bitmap
                sqrt_price_x64,
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

        CommandsName::OpenPosition {
            tick_lower_price,
            tick_upper_price,
            is_base_0,
            input_amount,
            with_metadata,
        } => {
            println!("Opening position with parameters:");
            println!("- Lower price: {}", tick_lower_price);
            println!("- Upper price: {}", tick_upper_price);
            println!("- Is base token 0: {}", is_base_0);
            println!("- Input amount: {}", input_amount);
            println!("- With metadata: {}", with_metadata);
            

            println!("pool_config.pool_id_account: {}", pool_config.pool_id_account.unwrap());

            // load pool to get observation
            let pool: fun_uniswap_v3::states::PoolState =
                program.account(pool_config.pool_id_account.unwrap())?;
            
            // Copy packed fields to local variables for safe access
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let tick_current = pool.tick_current;
            let tick_spacing = pool.tick_spacing;
            let token_mint_0 = pool.token_mint_0;
            let token_mint_1 = pool.token_mint_1;

            println!("\nPool information:");
            println!("- Pool ID: {}", pool_config.pool_id_account.unwrap());
            println!("- Token 0: {}", token_mint_0);
            println!("- Token 1: {}", token_mint_1);
            println!("- Current sqrt price: {}", sqrt_price_x64);
            println!("- Current tick: {}", tick_current);

            println!("pool.mint_decimals_0: {}", pool.mint_decimals_0);
            println!("pool.mint_decimals_1: {}", pool.mint_decimals_1);

            // 计算 tickLower/tickUpper
            let tick_lower_price_x64 = price_to_sqrt_price_x64(
                tick_lower_price,
                pool.mint_decimals_0,
                pool.mint_decimals_1,
            );
            let tick_upper_price_x64 = price_to_sqrt_price_x64(
                tick_upper_price,
                pool.mint_decimals_0,
                pool.mint_decimals_1,
            );

            println!("tick_lower_price_x64: {}", tick_lower_price_x64);
            println!("tick_upper_price_x64: {}", tick_upper_price_x64);

            let tick_lower_index = tick_with_spacing(
                tick_math::get_tick_at_sqrt_price(tick_lower_price_x64)?,
                tick_spacing.into(),
            );
            let tick_upper_index = tick_with_spacing(
                tick_math::get_tick_at_sqrt_price(tick_upper_price_x64)?,
                tick_spacing.into(),
            );

            println!(
                "tick_lower_index:{}, tick_upper_index:{}",
                tick_lower_index, tick_upper_index
            );

              // load position
              let position_nft_infos = get_all_nft_and_position_by_owner(
                &rpc_client,
                &payer.pubkey(),
                &pool_config.raydium_v3_program,
            );
            println!("\nFound {} existing positions", position_nft_infos.len());
            
            

        }



    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PositionNftTokenInfo {
    key: Pubkey,
    program: Pubkey,
    position: Pubkey,
    mint: Pubkey,
    amount: u64,
    decimals: u8,
}
fn get_all_nft_and_position_by_owner(
    client: &RpcClient,
    owner: &Pubkey,
    raydium_amm_v3_program: &Pubkey,
) -> Vec<PositionNftTokenInfo> {
    let mut spl_nfts = get_nft_account_and_position_by_owner(
        client,
        owner,
        spl_token::id(),
        raydium_amm_v3_program,
    );
    let spl_2022_nfts = get_nft_account_and_position_by_owner(
        client,
        owner,
        spl_token_2022::id(),
        raydium_amm_v3_program,
    );
    spl_nfts.extend(spl_2022_nfts);
    spl_nfts
}
fn get_nft_account_and_position_by_owner(
    client: &RpcClient,
    owner: &Pubkey,
    token_program: Pubkey,
    raydium_amm_v3_program: &Pubkey,
) -> Vec<PositionNftTokenInfo> {
    // println!("client: {:}", client);
    println!("owner: {:?}", owner);
    println!("token_program: {:?}", token_program);
    println!("raydium_amm_v3_program: {:?}", raydium_amm_v3_program);
    let all_tokens = client
        .get_token_accounts_by_owner(owner, TokenAccountsFilter::ProgramId(token_program))
        .unwrap();
    println!("all_tokens: {:?}", all_tokens);
    let mut position_nft_accounts = Vec::new();
    for keyed_account in all_tokens {
        if let UiAccountData::Json(parsed_account) = keyed_account.account.data {
            if parsed_account.program == "spl-token" || parsed_account.program == "spl-token-2022" {
                if let Ok(TokenAccountType::Account(ui_token_account)) =
                    serde_json::from_value(parsed_account.parsed)
                {
                    let _frozen = ui_token_account.state == UiAccountState::Frozen;

                    let token = ui_token_account
                        .mint
                        .parse::<Pubkey>()
                        .unwrap_or_else(|err| panic!("Invalid mint: {}", err));
                    let token_account = keyed_account
                        .pubkey
                        .parse::<Pubkey>()
                        .unwrap_or_else(|err| panic!("Invalid token account: {}", err));
                    let token_amount = ui_token_account
                        .token_amount
                        .amount
                        .parse::<u64>()
                        .unwrap_or_else(|err| panic!("Invalid token amount: {}", err));

                    let _close_authority = ui_token_account.close_authority.map_or(*owner, |s| {
                        s.parse::<Pubkey>()
                            .unwrap_or_else(|err| panic!("Invalid close authority: {}", err))
                    });

                    if ui_token_account.token_amount.decimals == 0 && token_amount == 1 {
                        let (position_pda, _) = Pubkey::find_program_address(
                            &[
                                fun_uniswap_v3::states::POSITION_SEED.as_bytes(),
                                token.to_bytes().as_ref(),
                            ],
                            &raydium_amm_v3_program,
                        );
                        position_nft_accounts.push(PositionNftTokenInfo {
                            key: token_account,
                            program: token_program,
                            position: position_pda,
                            mint: token,
                            amount: token_amount,
                            decimals: ui_token_account.token_amount.decimals,
                        }
                    );
                    }
                }
            }
        }
    }
    position_nft_accounts
}