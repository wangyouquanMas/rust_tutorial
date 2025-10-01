use anchor_lang::prelude::AccountMeta;
use anchor_client::{Client, Cluster};
use solana_sdk::{
    pubkey::Pubkey, instruction::Instruction,system_program,sysvar
};
use anyhow::Result;
use super::super::{read_keypair_file, ClientConfig};
use std::rc::Rc;
use fun_uniswap_v3::states::{  AMM_CONFIG_SEED, OBSERVATION_SEED, OPERATION_SEED, POOL_SEED, POOL_VAULT_SEED, POSITION_SEED,
    TICK_ARRAY_SEED, POOL_TICK_ARRAY_BITMAP_SEED,};
use fun_uniswap_v3::accounts;
use fun_uniswap_v3::instruction;
use fun_uniswap_v3::accounts::OpenPositionWithToken22Nft;



pub fn create_amm_config_instr(
    config: &ClientConfig,
    config_index: u16,
    tick_spacing: u16,
    trade_fee_rate: u32,
    protocol_fee_rate: u32,
    fund_fee_rate: u32,
) -> Result<Vec<Instruction>> {
    let payer = read_keypair_file(&config.admin_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_v3_program)?;
    let (amm_config_key, __bump) = Pubkey::find_program_address(
        &[AMM_CONFIG_SEED.as_bytes(), &config_index.to_be_bytes()],
        &program.id(),
    );
    let instructions = program
        .request()
        .accounts(fun_uniswap_v3::accounts::CreateAmmConfig {
            owner: program.payer(),
            amm_config: amm_config_key,
            system_program: system_program::id(),
        })
        .args(fun_uniswap_v3::instruction::CreateAmmConfig {
            index: config_index,
            tick_spacing,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
        })
        .instructions()?;
    Ok(instructions)
}


pub fn create_pool_instr(
    config: &ClientConfig,
    amm_config: Pubkey,
    token_mint_0: Pubkey,
    token_mint_1: Pubkey,
    token_program_0: Pubkey,
    token_program_1: Pubkey,
    tick_array_bitmap: Pubkey,
    sqrt_price_x64: u128,
    open_time: u64,
) -> Result<Vec<Instruction>> {
    let payer = read_keypair_file(&config.payer_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_v3_program)?;
    
    // DEBUG: Print the input parameters
    println!("DEBUG create_pool_instr:");
    println!("  amm_config: {}", amm_config);
    println!("  token_mint_0: {}", token_mint_0);
    println!("  token_mint_1: {}", token_mint_1);
    println!("  tick_array_bitmap (passed): {}", tick_array_bitmap);
    
    let (pool_account_key, pool_bump) = Pubkey::find_program_address(
        &[
            POOL_SEED.as_bytes(),
            amm_config.to_bytes().as_ref(),
            token_mint_0.to_bytes().as_ref(),
            token_mint_1.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    println!("  pool_account_key: {} (bump: {})", pool_account_key, pool_bump);
    
    // DEBUG: Calculate bitmap extension ourselves and compare
    let (calculated_bitmap, bitmap_bump) = Pubkey::find_program_address(
        &[
            POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    println!("  calculated_bitmap: {} (bump: {})", calculated_bitmap, bitmap_bump);
    println!("  bitmap match: {}", calculated_bitmap == tick_array_bitmap);
    
    let (token_vault_0, vault0_bump) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            token_mint_0.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    println!("  token_vault_0: {} (bump: {})", token_vault_0, vault0_bump);
    
    let (token_vault_1, vault1_bump) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            token_mint_1.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    println!("  token_vault_1: {} (bump: {})", token_vault_1, vault1_bump);
    
    let (observation_key, obs_bump) = Pubkey::find_program_address(
        &[
            OBSERVATION_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    println!("  observation_key: {} (bump: {})", observation_key, obs_bump);
    
    let instructions = program
        .request()
        .accounts(accounts::CreatePool {
            pool_creator: program.payer(),
            amm_config,
            pool_state: pool_account_key,
            token_mint_0,
            token_mint_1,
            token_vault_0,
            token_vault_1,
            observation_state: observation_key,
            tick_array_bitmap,
            token_program_0,
            token_program_1,
            system_program: system_program::id(),
            rent: sysvar::rent::id(),
        })
        .args(instruction::CreatePool {
            sqrt_price_x64,
            open_time,
        })
        .instructions()?;
    Ok(instructions)
}


pub fn open_position_with_token22_nft_instr(
    config: &ClientConfig,
    pool_account_key: Pubkey,
    token_vault_0: Pubkey,
    token_vault_1: Pubkey,
    token_mint_0: Pubkey,
    token_mint_1: Pubkey,
    nft_mint_key: Pubkey,
    nft_to_owner: Pubkey,
    user_token_account_0: Pubkey,
    user_token_account_1: Pubkey,
    remaining_accounts: Vec<AccountMeta>,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    with_metadata: bool,
) -> Result<Vec<Instruction>> {

    println!("open_position_with_token22_nft_instr liquidity: {:?}", liquidity);


    let payer = read_keypair_file(&config.payer_path)?;
    let url = Cluster::Custom(config.http_url.clone(), config.ws_url.clone());
    // Client.
    let client = Client::new(url, Rc::new(payer));
    let program = client.program(config.raydium_v3_program)?;
    let nft_ata_token_account =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &program.payer(),
            &nft_mint_key,
            &spl_token_2022::id(),
        );
    let (protocol_position_key, __bump) = Pubkey::find_program_address(
        &[
            POSITION_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            &tick_lower_index.to_be_bytes(),
            &tick_upper_index.to_be_bytes(),
        ],
        &program.id(),
    );
    let (tick_array_lower, __bump) = Pubkey::find_program_address(
        &[
            TICK_ARRAY_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            &tick_array_lower_start_index.to_be_bytes(),
        ],
        &program.id(),
    );
    let (tick_array_upper, __bump) = Pubkey::find_program_address(
        &[
            TICK_ARRAY_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            &tick_array_upper_start_index.to_be_bytes(),
        ],
        &program.id(),
    );
    let (personal_position_key, __bump) = Pubkey::find_program_address(
        &[POSITION_SEED.as_bytes(), nft_mint_key.to_bytes().as_ref()],
        &program.id(),
    );
    let instructions = program
        .request()
        .accounts(OpenPositionWithToken22Nft {
            payer: program.payer(),
            position_nft_owner: nft_to_owner,
            position_nft_mint: nft_mint_key,
            position_nft_account: nft_ata_token_account,
            pool_state: pool_account_key,
            protocol_position: protocol_position_key,
            tick_array_lower,
            tick_array_upper,
            personal_position: personal_position_key,
            token_account_0: user_token_account_0,
            token_account_1: user_token_account_1,
            token_vault_0,
            token_vault_1,
            rent: sysvar::rent::id(),
            system_program: system_program::id(),
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::id(),
            token_program_2022: spl_token_2022::id(),
            vault_0_mint: token_mint_0,
            vault_1_mint: token_mint_1,
        })
        .accounts(remaining_accounts)
        .args(fun_uniswap_v3::instruction::OpenPositionWithToken22Nft {
            liquidity,
            amount_0_max,
            amount_1_max,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            with_metadata,
            base_flag: None,
        })
        .instructions()?;
    Ok(instructions)
}
