use anchor_client::{Client, Cluster};
use solana_sdk::{
    pubkey::Pubkey, instruction::Instruction,system_program,sysvar
};
use anyhow::Result;
use super::super::{read_keypair_file, ClientConfig};
use std::rc::Rc;
use fun_uniswap_v3::states::AMM_CONFIG_SEED;
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

