use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::state::{AmmConfig, AMM_CONFIG_SEED, FEE_RATE_DENOMINATOR, MAX_FUND_FEE_RATE, MAX_PROTOCOL_FEE_RATE, MAX_TRADE_FEE_RATE};

/// Accounts required to initialize a new global AMM configuration.
#[derive(Accounts)]
#[instruction(index: u16)]
pub struct InitializeAmmConfig<'info> {
    /// Funds the rent-exempt allocation of `amm_config`.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Designated authority allowed to manage AMM-wide settings.
    pub authority: Signer<'info>,

    /// PDA that stores global settings shared by all pools created by `authority`.
    #[account(
        init,
        seeds = [AMM_CONFIG_SEED.as_bytes(), &index.to_be_bytes()],
        bump,
        payer = payer,
        space = AmmConfig::space(),
    )]
    pub amm_config: Account<'info, AmmConfig>,

    /// Required by Anchor to create the account using the payer's lamports.
    pub system_program: Program<'info, System>,
}

pub fn initialize_amm_config(
    ctx: Context<InitializeAmmConfig>,
    _index: u16,
    tick_spacing: u16,
    trade_fee_rate: u32,
    protocol_fee_rate: u32,
    fund_fee_rate: u32,
) -> Result<()> {
    require!(tick_spacing > 0, ErrorCode::InvalidFeeTier);
    require!(trade_fee_rate <= MAX_TRADE_FEE_RATE, ErrorCode::InvalidTradeFeeRate);
    require!(protocol_fee_rate <= MAX_PROTOCOL_FEE_RATE, ErrorCode::InvalidProtocolFeeRate);
    require!(fund_fee_rate <= MAX_FUND_FEE_RATE, ErrorCode::InvalidFundFeeRate);

    let total_fee_rate = trade_fee_rate
        .checked_add(protocol_fee_rate)
        .and_then(|val| val.checked_add(fund_fee_rate))
        .ok_or(ErrorCode::TotalFeeRateTooHigh)?;
    require!(total_fee_rate <= FEE_RATE_DENOMINATOR, ErrorCode::TotalFeeRateTooHigh);

    let amm_config = &mut ctx.accounts.amm_config;
    amm_config.bump = *ctx.bumps.get("amm_config").unwrap_or(&0);
    amm_config.authority = ctx.accounts.authority.key();
    amm_config.tick_spacing = tick_spacing;
    amm_config.trade_fee_rate = trade_fee_rate;
    amm_config.protocol_fee_rate = protocol_fee_rate;
    amm_config.fund_fee_rate = fund_fee_rate;

    Ok(())
}

