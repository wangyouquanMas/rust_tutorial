use anchor_lang::prelude::*;

use crate::state::{AmmConfig, AMM_CONFIG_SEED};

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

/// TODO(step3): implement initialization logic & validations.
pub fn initialize_amm_config(
    _ctx: Context<InitializeAmmConfig>,
    _index: u16,
    _tick_spacing: u16,
    _trade_fee_rate: u32,
    _protocol_fee_rate: u32,
    _fund_fee_rate: u32,
) -> Result<()> {
    Ok(())
}

