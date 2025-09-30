use anchor_lang::prelude::*;
pub mod states;
pub mod instructions;
pub mod errors;
pub mod error;
pub mod events;
pub mod util;
pub mod libraries;

use instructions::*;
use core as core_;

declare_id!("SxbynnGn1WVBXUjdY3uBBCSgF6pX8Fpydn9VZcCnVHC");


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

        /// Creates a new position wrapped in a Token2022 NFT without relying on metadata_program and metadata_account, reduce the cost for user to create a personal position.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of accounts
    /// * `tick_lower_index` - The low boundary of market
    /// * `tick_upper_index` - The upper boundary of market
    /// * `tick_array_lower_start_index` - The start index of tick array which include tick low
    /// * `tick_array_upper_start_index` - The start index of tick array which include tick upper
    /// * `liquidity` - The liquidity to be added, if zero, and the base_flag is specified, calculate liquidity base amount_0_max or amount_1_max according base_flag, otherwise open position with zero liquidity
    /// * `amount_0_max` - The max amount of token_0 to spend, which serves as a slippage check
    /// * `amount_1_max` - The max amount of token_1 to spend, which serves as a slippage check
    /// * `with_metadata` - The flag indicating whether to create NFT mint metadata
    /// * `base_flag` - if the liquidity specified as zero, true: calculate liquidity base amount_0_max otherwise base amount_1_max
    ///
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

       /// #[deprecated(note = "Use `swap_v2` instead.")]
    /// Swaps one token for as much as possible of another token across a single pool
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of accounts
    /// * `amount` - Arranged in pairs with other_amount_threshold. (amount_in, amount_out_minimum) or (amount_out, amount_in_maximum)
    /// * `other_amount_threshold` - For slippage check
    /// * `sqrt_price_limit` - The Q64.64 sqrt price √P limit. If zero for one, the price cannot
    /// * `is_base_input` - swap base input or swap base output
    ///
    pub fn swap<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SwapSingle<'info>>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit_x64: u128,
        is_base_input: bool,
    ) -> Result<()> {
        instructions::swap(
            ctx,
            amount,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input,
        )
    }
}

#[derive(Accounts)]
pub struct Initialize {}
