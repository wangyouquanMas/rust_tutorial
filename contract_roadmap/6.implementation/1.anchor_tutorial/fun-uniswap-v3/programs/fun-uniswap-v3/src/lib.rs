use anchor_lang::prelude::*;
pub mod states;
pub mod instructions;
pub mod error;
pub mod events;
pub mod util;
pub mod libraries;

use instructions::*;
pub use core as core_;

declare_id!("DVCq6TyPx1Xfy22NPFDNVkpyk7PzozpsFJCmbBjCCpSd");


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

    pub fn create_pool(
        ctx: Context<CreatePool>,
        sqrt_price_x64: u128,
        open_time: u64,
    ) -> Result<()> {
        instructions::create_pool::create_pool(ctx, sqrt_price_x64, open_time)
    }

    pub fn open_position_with_token22_nft<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, OpenPositionWithToken22Nft<'info>>,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
        with_metadata: bool,
        base_flag: Option<bool>,
    ) -> Result<()> {
        instructions::open_position_with_token22_nft(
            ctx,
            liquidity,
            amount_0_max,
            amount_1_max,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            with_metadata,
            base_flag,
        )
    }
}

#[derive(Accounts)]
pub struct Initialize {}