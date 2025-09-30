use arrayref::array_ref;
use anchor_client::{Client, Cluster};
use anchor_lang::prelude::AccountMeta;
use anyhow::{format_err, Result};
use solana_client::{
    rpc_client::RpcClient,
    rpc_request::TokenAccountsFilter,
};
use clap::Parser; 
use fun_uniswap_v3::{
    libraries::{tick_math, liquidity_math},
    states::{PoolState, TickArrayBitmapExtension, TickArrayState,POOL_SEED,POOL_TICK_ARRAY_BITMAP_SEED},
};
use solana_account_decoder::{
    parse_token::{TokenAccountType, UiAccountState},
    UiAccountData, UiAccountEncoding,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair,Signer},
    system_program,
    transaction::Transaction,
    program_pack::Pack,
    compute_budget::ComputeBudgetInstruction,
    commitment_config::CommitmentConfig,
};
use configparser::ini::Ini;
use std::rc::Rc;
use spl_token_2022::{
    extension::StateWithExtensions,
    state::Mint,
    state::{Account, AccountState},
};
use std::str::FromStr;

mod instructions;
use instructions::utils::*;
use instructions::amm_instructions::*;
use instructions::rpc::*;

use crate::instructions::utils;
use std::{collections::VecDeque, convert::identity, mem::size_of};


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
    Swap {
        #[arg(long)]
        input_token: Pubkey,
        #[arg(long)]
        output_token: Pubkey,
        #[arg(long)]
        base_in: bool,
        #[arg(long)]
        simulate: bool,
        #[arg(long)]
        amount: u64,
        #[arg(long)]
        limit_price: Option<f64>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct PositionNftTokenInfo {
    key: Pubkey,
    program: Pubkey,
    position: Pubkey,
    mint: Pubkey,
    amount: u64,
    decimals: u8,
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

            // 新增详细日志
            println!("\n[DEBUG] Key CLMM Params:");
            println!("tickSpacing: {}", tick_spacing);
            println!("tickArraySize: {}", fun_uniswap_v3::states::TICK_ARRAY_SIZE);
            println!("tickLower (index): {}", tick_lower_index);
            println!("tickUpper (index): {}", tick_upper_index);

            let tick_array_lower_start_index =
                fun_uniswap_v3::states::TickArrayState::get_array_start_index(
                    tick_lower_index,
                    tick_spacing.into(),
                );
            let tick_array_upper_start_index =
                fun_uniswap_v3::states::TickArrayState::get_array_start_index(
                    tick_upper_index,
                    tick_spacing.into(),
                );
            println!("tickArrayLowerStartIndex: {}", tick_array_lower_start_index);
            println!("tickArrayUpperStartIndex: {}", tick_array_upper_start_index);

            println!("\nTick array indices:");
            println!("- Lower array start index: {}", tick_array_lower_start_index);
            println!("- Upper array start index: {}", tick_array_upper_start_index);
            
            let tick_lower_price_x64 = tick_math::get_sqrt_price_at_tick(tick_lower_index)?;
            let tick_upper_price_x64 = tick_math::get_sqrt_price_at_tick(tick_upper_index)?;

            println!("tick_lower_price_x64: {}", tick_lower_price_x64);
            println!("tick_upper_price_x64: {}", tick_upper_price_x64);

            let liquidity = if is_base_0 {
                println!("is base 0");
                liquidity_math::get_liquidity_from_single_amount_0(
                    pool.sqrt_price_x64,
                    tick_lower_price_x64,
                    tick_upper_price_x64,
                    input_amount,
                )
            } else {
                liquidity_math::get_liquidity_from_single_amount_1(
                    pool.sqrt_price_x64,
                    tick_lower_price_x64,
                    tick_upper_price_x64,
                    input_amount,
                )
            };
            let (amount_0, amount_1) = liquidity_math::get_delta_amounts_signed(
                pool.tick_current,
                pool.sqrt_price_x64,
                tick_lower_index,
                tick_upper_index,
                liquidity as i128,
            )?;
            println!("\nCalculated amounts:");
            println!("- Amount 0: {}", amount_0);
            println!("- Amount 1: {}", amount_1);
            println!("- Liquidity: {}", liquidity);
            
            // calc with slippage
            let amount_0_with_slippage =
                amount_with_slippage(amount_0 as u64, pool_config.slippage, true);
            let amount_1_with_slippage =
                amount_with_slippage(amount_1 as u64, pool_config.slippage, true);
            println!("\nAmounts with slippage:");
            println!("- Amount 0 with slippage: {}", amount_0_with_slippage);
            println!("- Amount 1 with slippage: {}", amount_1_with_slippage);
            
            // calc with transfer_fee
            let transfer_fee = get_pool_mints_inverse_fee(
                &rpc_client,
                pool.token_mint_0,
                pool.token_mint_1,
                amount_0_with_slippage,
                amount_1_with_slippage,
            );
            println!("\nTransfer fees:");
            println!("- Transfer fee 0: {}", transfer_fee.0.transfer_fee);
            println!("- Transfer fee 1: {}", transfer_fee.1.transfer_fee);
            
            let amount_0_max = (amount_0_with_slippage as u64)
                .checked_add(transfer_fee.0.transfer_fee)
                .unwrap();
            let amount_1_max = (amount_1_with_slippage as u64)
                .checked_add(transfer_fee.1.transfer_fee)
                .unwrap();
            println!("\nMaximum amounts (including fees):");
            println!("- Max amount 0: {}", amount_0_max);
            println!("- Max amount 1: {}", amount_1_max);

            let tick_array_lower_start_index =
                fun_uniswap_v3::states::TickArrayState::get_array_start_index(
                    tick_lower_index,
                    tick_spacing.into(),
                );
            let tick_array_upper_start_index =
                fun_uniswap_v3::states::TickArrayState::get_array_start_index(
                    tick_upper_index,
                    tick_spacing.into(),
                );
            println!("\nTick array indices:");
            println!("- Lower array start index: {}", tick_array_lower_start_index);
            println!("- Upper array start index: {}", tick_array_upper_start_index);
            
            // load position
            let position_nft_infos = get_all_nft_and_position_by_owner(
                &rpc_client,
                &payer.pubkey(),
                &pool_config.raydium_v3_program,
            );
            println!("\nFound {} existing positions", position_nft_infos.len());
            
            let positions: Vec<Pubkey> = position_nft_infos
                .iter()
                .map(|item| item.position)
                .collect();
            let rsps = rpc_client.get_multiple_accounts(&positions)?;
            let mut user_positions = Vec::new();
            for rsp in rsps {
                match rsp {
                    None => continue,
                    Some(rsp) => {
                        let position = deserialize_anchor_account::<
                            fun_uniswap_v3::states::PersonalPositionState,
                        >(&rsp)?;
                        user_positions.push(position);
                    }
                }
            }
            let mut find_position = fun_uniswap_v3::states::PersonalPositionState::default();
            for position in user_positions {
                if position.pool_id == pool_config.pool_id_account.unwrap()
                    && position.tick_lower_index == tick_lower_index
                    && position.tick_upper_index == tick_upper_index
                {
                    find_position = position.clone();
                }
            }
            if find_position.nft_mint == Pubkey::default() {
                println!("\nNo existing position found, creating new position...");
                // personal position not exist
                // new nft mint
                let nft_mint = Keypair::new();
                println!("- New NFT mint address: {}", nft_mint.pubkey());
                
                let mut remaining_accounts = Vec::new();
                remaining_accounts.push(AccountMeta::new(
                    pool_config.tickarray_bitmap_extension.unwrap(),
                    false,
                ));
                println!("- Tick array bitmap extension: {}", pool_config.tickarray_bitmap_extension.unwrap());

                let mut instructions = Vec::new();
                let request_inits_instr =
                    ComputeBudgetInstruction::set_compute_unit_limit(1400_000u32);
                instructions.push(request_inits_instr);
                let open_position_instr = open_position_with_token22_nft_instr(
                    &pool_config.clone(),
                    pool_config.pool_id_account.unwrap(),
                    pool.token_vault_0,
                    pool.token_vault_1,
                    pool.token_mint_0,
                    pool.token_mint_1,
                    nft_mint.pubkey(),
                    payer.pubkey(),
                    spl_associated_token_account::get_associated_token_address_with_program_id(
                        &payer.pubkey(),
                        &pool_config.mint0.unwrap(),
                        &transfer_fee.0.owner,
                    ),
                    spl_associated_token_account::get_associated_token_address_with_program_id(
                        &payer.pubkey(),
                        &pool_config.mint1.unwrap(),
                        &transfer_fee.1.owner,
                    ),
                    remaining_accounts,
                    liquidity,
                    amount_0_max,
                    amount_1_max,
                    tick_lower_index,
                    tick_upper_index,
                    tick_array_lower_start_index,
                    tick_array_upper_start_index,
                    with_metadata,
                )?;
                instructions.extend(open_position_instr);
                println!("\nSending transaction...");
                // send
                let signers = vec![&payer, &nft_mint];
                let recent_hash = rpc_client.get_latest_blockhash()?;
                let txn = Transaction::new_signed_with_payer(
                    &instructions,
                    Some(&payer.pubkey()),
                    &signers,
                    recent_hash,
                );
                let signature = send_txn(&rpc_client, &txn, true)?;
                println!("Transaction signature: {}", signature);
            } else {
                // personal position exist
                println!("\nPosition already exists:");
                println!("- NFT mint: {}", find_position.nft_mint);
                println!("- Pool ID: {}", find_position.pool_id);
                println!("- Lower tick: {}", find_position.tick_lower_index);
                println!("- Upper tick: {}", find_position.tick_upper_index);
            }
        }
    CommandsName::Swap {
        input_token,
        output_token,
        base_in,
        simulate,
        amount,
        limit_price,
    } => {
        // load mult account
        let load_accounts = vec![
            input_token,
            output_token,
            pool_config.amm_config_key,
            pool_config.pool_id_account.unwrap(),
            pool_config.tickarray_bitmap_extension.unwrap(),
        ];
        let rsps = rpc_client.get_multiple_accounts(&load_accounts)?;
        let [user_input_account, user_output_account, amm_config_account, pool_account, tickarray_bitmap_extension_account] =
            array_ref![rsps, 0, 5];
        let user_input_state =
            StateWithExtensions::<Account>::unpack(&user_input_account.as_ref().unwrap().data)
                .unwrap();
        let user_output_state =
            StateWithExtensions::<Account>::unpack(&user_output_account.as_ref().unwrap().data)
                .unwrap();
        let amm_config_state = deserialize_anchor_account::<fun_uniswap_v3::states::AmmConfig>(
            amm_config_account.as_ref().unwrap(),
        )?;
        let pool_state = deserialize_anchor_account::<fun_uniswap_v3::states::PoolState>(
            pool_account.as_ref().unwrap(),
        )?;
        let tickarray_bitmap_extension =
            deserialize_anchor_account::<fun_uniswap_v3::states::TickArrayBitmapExtension>(
                tickarray_bitmap_extension_account.as_ref().unwrap(),
            )?;

        println!("tickarray_bitmap_extension:{:?}", tickarray_bitmap_extension);
        let zero_for_one = user_input_state.base.mint == pool_state.token_mint_0;
        // load tick_arrays
        let mut tick_arrays = load_cur_and_next_five_tick_array(
            &rpc_client,
            &pool_config,
            &pool_state,
            &tickarray_bitmap_extension,
            zero_for_one,
        );

        let mut sqrt_price_limit_x64 = None;
        if limit_price.is_some() {
            let sqrt_price_x64 = price_to_sqrt_price_x64(
                limit_price.unwrap(),
                pool_state.mint_decimals_0,
                pool_state.mint_decimals_1,
            );
            sqrt_price_limit_x64 = Some(sqrt_price_x64);
        }

        let (mut other_amount_threshold, mut tick_array_indexs) =
            utils::get_out_put_amount_and_remaining_accounts(
                amount,
                sqrt_price_limit_x64,
                zero_for_one,
                base_in,
                &amm_config_state,
                &pool_state,
                &tickarray_bitmap_extension,
                &mut tick_arrays,
            )
            .unwrap();
        println!(
            "amount:{}, other_amount_threshold:{}",
            amount, other_amount_threshold
        );
        if base_in {
            // min out
            other_amount_threshold =
                amount_with_slippage(other_amount_threshold, pool_config.slippage, false);
        } else {
            // max in
            other_amount_threshold =
                amount_with_slippage(other_amount_threshold, pool_config.slippage, true);
        }

        let current_or_next_tick_array_key = Pubkey::find_program_address(
            &[
                fun_uniswap_v3::states::TICK_ARRAY_SEED.as_bytes(),
                pool_config.pool_id_account.unwrap().to_bytes().as_ref(),
                &tick_array_indexs.pop_front().unwrap().to_be_bytes(),
            ],
            &pool_config.raydium_v3_program,
        )
        .0;
        let mut remaining_accounts = Vec::new();
        remaining_accounts.push(AccountMeta::new_readonly(
            pool_config.tickarray_bitmap_extension.unwrap(),
            false,
        ));
        let mut accounts = tick_array_indexs
            .into_iter()
            .map(|index| {
                AccountMeta::new(
                    Pubkey::find_program_address(
                        &[
                            fun_uniswap_v3::states::TICK_ARRAY_SEED.as_bytes(),
                            pool_config.pool_id_account.unwrap().to_bytes().as_ref(),
                            &index.to_be_bytes(),
                        ],
                        &pool_config.raydium_v3_program,
                    )
                    .0,
                    false,
                )
            })
            .collect();
        remaining_accounts.append(&mut accounts);
        let mut instructions = Vec::new();
        let request_inits_instr = ComputeBudgetInstruction::set_compute_unit_limit(1400_000u32);
        instructions.push(request_inits_instr);
        let swap_instr = swap_instr(
            &pool_config.clone(),
            pool_state.amm_config,
            pool_config.pool_id_account.unwrap(),
            if zero_for_one {
                pool_state.token_vault_0
            } else {
                pool_state.token_vault_1
            },
            if zero_for_one {
                pool_state.token_vault_1
            } else {
                pool_state.token_vault_0
            },
            pool_state.observation_key,
            input_token,
            output_token,
            current_or_next_tick_array_key,
            remaining_accounts,
            amount,
            other_amount_threshold,
            sqrt_price_limit_x64,
            base_in,
        )
        .unwrap();
        instructions.extend(swap_instr);
        // send
        let signers = vec![&payer];
        let recent_hash = rpc_client.get_latest_blockhash()?;
        let txn = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &signers,
            recent_hash,
        );
        if simulate {
            let ret =
                simulate_transaction(&rpc_client, &txn, true, CommitmentConfig::confirmed())?;
            println!("{:#?}", ret);
        } else {
            let signature = send_txn(&rpc_client, &txn, true)?;
            println!("{}", signature);
        }
     }
    }
    Ok(())
}

fn get_all_nft_and_position_by_owner(
    client: &RpcClient,
    owner: &Pubkey,
    fun_uniswap_v3_program: &Pubkey,
) -> Vec<PositionNftTokenInfo> {
    let mut spl_nfts = get_nft_account_and_position_by_owner(
        client,
        owner,
        spl_token::id(),
        fun_uniswap_v3_program,
    );
    let spl_2022_nfts = get_nft_account_and_position_by_owner(
        client,
        owner,
        spl_token_2022::id(),
        fun_uniswap_v3_program,
    );
    spl_nfts.extend(spl_2022_nfts);
    spl_nfts
}
fn get_nft_account_and_position_by_owner(
    client: &RpcClient,
    owner: &Pubkey,
    token_program: Pubkey,
    fun_uniswap_v3_program: &Pubkey,
) -> Vec<PositionNftTokenInfo> {
    // println!("client: {:}", client);
    println!("owner: {:?}", owner);
    println!("token_program: {:?}", token_program);
    println!("fun_uniswap_v3_program: {:?}", fun_uniswap_v3_program);
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
                            &fun_uniswap_v3_program,
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



fn load_cur_and_next_five_tick_array(
    rpc_client: &RpcClient,
    pool_config: &ClientConfig,
    pool_state: &PoolState,
    tickarray_bitmap_extension: &TickArrayBitmapExtension,
    zero_for_one: bool,
) -> VecDeque<TickArrayState> {
    let (_, mut current_vaild_tick_array_start_index) = pool_state
        .get_first_initialized_tick_array(&Some(*tickarray_bitmap_extension), zero_for_one)
        .unwrap();
    let mut tick_array_keys = Vec::new();
    tick_array_keys.push(
        Pubkey::find_program_address(
            &[
                fun_uniswap_v3::states::TICK_ARRAY_SEED.as_bytes(),
                pool_config.pool_id_account.unwrap().to_bytes().as_ref(),
                &current_vaild_tick_array_start_index.to_be_bytes(),
            ],
            &pool_config.raydium_v3_program,
        )
        .0,
    );
    let mut max_array_size = 5;
    while max_array_size != 0 {
        let next_tick_array_index = pool_state
            .next_initialized_tick_array_start_index(
                &Some(*tickarray_bitmap_extension),
                current_vaild_tick_array_start_index,
                zero_for_one,
            )
            .unwrap();
        if next_tick_array_index.is_none() {
            break;
        }
        current_vaild_tick_array_start_index = next_tick_array_index.unwrap();
        tick_array_keys.push(
            Pubkey::find_program_address(
                &[
                    fun_uniswap_v3::states::TICK_ARRAY_SEED.as_bytes(),
                    pool_config.pool_id_account.unwrap().to_bytes().as_ref(),
                    &current_vaild_tick_array_start_index.to_be_bytes(),
                ],
                &pool_config.raydium_v3_program,
            )
            .0,
        );
        max_array_size -= 1;
    }
    let tick_array_rsps = rpc_client.get_multiple_accounts(&tick_array_keys).unwrap();
    let mut tick_arrays = VecDeque::new();
    for tick_array in tick_array_rsps {
        let tick_array_state =
            deserialize_anchor_account::<fun_uniswap_v3::states::TickArrayState>(
                &tick_array.unwrap(),
            )
            .unwrap();
        tick_arrays.push_back(tick_array_state);
    }
    tick_arrays
}