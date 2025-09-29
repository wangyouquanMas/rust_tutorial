use crate::state::{AmmConfig, PoolState};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

/// Accounts required to create and initialize a new pool.
#[derive(Accounts)]
pub struct CreatePool<'info> {
    /// Signer paying for all newly created accounts.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Global configuration that governs this pool.
    pub amm_config: Account<'info, AmmConfig>,

    /// Deterministic PDA storing core pool state data.
    #[account(
        init,
        payer = payer,
        space = PoolState::space(),
        seeds = pool_state_seeds(&amm_config.key(), &token_mint_0.key(), &token_mint_1.key()),
        bump,
    )]
    pub pool_state: Account<'info, PoolState>,

    /// Token mint with the smaller public key (enforced client side).
    #[account(mint::token_program = token_program_0)]
    pub token_mint_0: InterfaceAccount<'info, Mint>,

    /// Token mint with the larger public key.
    #[account(mint::token_program = token_program_1)]
    pub token_mint_1: InterfaceAccount<'info, Mint>,

    /// PDA token account controlled by `pool_state` for mint 0.
    #[account(
        mut,
        seeds = pool_vault_seeds(&pool_state.key(), &token_mint_0.key()),
        bump,
    )]
    pub token_vault_0: UncheckedAccount<'info>,

    /// PDA token account controlled by `pool_state` for mint 1.
    #[account(
        mut,
        seeds = pool_vault_seeds(&pool_state.key(), &token_mint_1.key()),
        bump,
    )]
    pub token_vault_1: UncheckedAccount<'info>,

    /// PDA storing oracle observations for TWAP calculations.
    #[account(
        init,
        payer = payer,
        space = 8,
        seeds = observation_seeds(&pool_state.key()),
        bump,
    )]
    pub observation_state: UncheckedAccount<'info>,

    /// PDA storing tick-array initialization bitmap.
    #[account(
        init,
        payer = payer,
        space = 8,
        seeds = tick_array_bitmap_seeds(&pool_state.key()),
        bump,
    )]
    pub tick_array_bitmap: UncheckedAccount<'info>,

    /// Token program for mint_0 (supports SPL Token or Token-2022).
    pub token_program_0: Interface<'info, TokenInterface>,
    /// Token program for mint_1.
    pub token_program_1: Interface<'info, TokenInterface>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,
    /// Rent sysvar required for account initialization.
    pub rent: Sysvar<'info, Rent>,
}

fn pool_state_seeds<'a>(
    amm_config: &'a Pubkey,
    token_mint_0: &'a Pubkey,
    token_mint_1: &'a Pubkey,
) -> [&'a [u8]; 4] {
    crate::state::pool_state_seeds(amm_config, token_mint_0, token_mint_1)
}

fn pool_vault_seeds<'a>(pool_state: &'a Pubkey, token_mint: &'a Pubkey) -> [&'a [u8]; 3] {
    crate::state::pool_vault_seeds(pool_state, token_mint)
}

fn observation_seeds<'a>(pool_state: &'a Pubkey) -> [&'a [u8]; 2] {
    crate::state::observation_seeds(pool_state)
}

fn tick_array_bitmap_seeds<'a>(pool_state: &'a Pubkey) -> [&'a [u8]; 2] {
    crate::state::tick_array_bitmap_seeds(pool_state)
}

