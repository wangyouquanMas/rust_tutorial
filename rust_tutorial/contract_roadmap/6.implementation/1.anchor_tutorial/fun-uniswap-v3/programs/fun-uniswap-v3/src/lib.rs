use anchor_lang::prelude::*;
pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;
pub mod utils;

use instructions::*;
pub use instructions::initialize_amm_config::InitializeAmmConfig;

declare_id!("HXdTUsTLztg64qJ8upPqHMnBAmeLA8Zmj1xUhDPqj4fn");

#[program]
pub mod fun_uniswap_v3 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn initialize_amm_config(
        ctx: Context<InitializeAmmConfig>,
        index: u16,
        tick_spacing: u16,
        trade_fee_rate: u32,
        protocol_fee_rate: u32,
        fund_fee_rate: u32,
    ) -> Result<()> {
        instructions::initialize_amm_config::initialize_amm_config(
            ctx,
            index,
            tick_spacing,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
        )
    }
}

#[derive(Accounts)]
pub struct Initialize {}
