use anchor_lang::prelude::*;
pub mod states;
pub mod instructions;
pub mod error;
pub mod events;
pub mod util;
pub mod libraries;

use instructions::*;
pub use core as core_;

declare_id!("7GagSvwxqA9cqpGCUfNH2xoEj1EkAw5NGSfqjhH7wBY8");


pub mod admin {
    use super::{pubkey, Pubkey};
    #[cfg(feature = "localnet")]
    pub const ID: Pubkey = pubkey!("3xbCoRgPcuUhUdsVJHrq79gmcGUT3VwqrHgMTkV296cP");
    #[cfg(not(feature = "localnet"))]
    pub const ID: Pubkey = pubkey!("3xbCoRgPcuUhUdsVJHrq79gmcGUT3VwqrHgMTkV296cP");
}

#[program]
pub mod fun_uniswap_v3 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn create_amm_config(
        ctx: Context<CreateAmmConfig>,
        index: u16,
        tick_spacing: u16,
        trade_fee_rate: u32,
        protocol_fee_rate: u32,
        fund_fee_rate: u32,
    ) -> Result<()> {
        instructions::admin::create_amm_config::create_amm_config(
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