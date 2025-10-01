use anchor_client::{Client, Cluster};
use solana_sdk::{
    pubkey::Pubkey, instruction::Instruction,system_program,sysvar
};
use anyhow::Result;
use super::super::{read_keypair_file, ClientConfig};
use std::rc::Rc;
use fun_uniswap_v3::states::{AMM_CONFIG_SEED, POOL_SEED, POOL_TICK_ARRAY_BITMAP_SEED, POOL_VAULT_SEED};
use fun_uniswap_v3::accounts;
use fun_uniswap_v3::instruction;



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
        token_program_0,
        token_program_1,
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    })
    .args(instruction::CreatePool {
        sqrt_price_x64,
    })
    .instructions()?;

    Ok(instructions)
}
