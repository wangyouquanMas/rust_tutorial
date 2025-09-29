use crate::errors::ErrorCode;
use crate::state::{self, AmmConfig, PoolState, TickArrayBitmapAccount, ObservationAccount};
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

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
        space = ObservationAccount::space(),
        seeds = observation_seeds(&pool_state.key()),
        bump,
    )]
    pub observation_state: Account<'info, ObservationAccount>,

    /// PDA storing tick-array initialization bitmap.
    #[account(
        init,
        payer = payer,
        space = TickArrayBitmapAccount::space(),
        seeds = tick_array_bitmap_seeds(&pool_state.key()),
        bump,
    )]
    pub tick_array_bitmap: Account<'info, TickArrayBitmapAccount>,

    /// Token program for mint_0 (supports SPL Token or Token-2022).
    pub token_program_0: Interface<'info, TokenInterface>,
    /// Token program for mint_1.
    pub token_program_1: Interface<'info, TokenInterface>,
    /// SPL Token program interface for CPI convenience (used when both mints are SPL token).
    pub token_program: Program<'info, Token>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,
    /// Rent sysvar required for account initialization.
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_pool(
    ctx: Context<CreatePool>,
    sqrt_price_x64: u128,
    tick_current: i32,
) -> Result<()> {
    let accounts = ctx.accounts;

    require!(
        accounts.token_mint_0.key() < accounts.token_mint_1.key(),
        ErrorCode::InvalidMintOrder
    );
    require!(
        state::tick_spacing_matches(&accounts.amm_config, accounts.amm_config.tick_spacing),
        ErrorCode::TickSpacingMismatch
    );

    let pool_bump = *ctx
        .bumps
        .get("pool_state")
        .ok_or(ErrorCode::MissingBump)?;

    let mut pool_state = accounts.pool_state.clone();
    pool_state.bump = pool_bump;
    pool_state.amm_config = accounts.amm_config.key();
    pool_state.authority = accounts.amm_config.authority;
    pool_state.token_mint_0 = accounts.token_mint_0.key();
    pool_state.token_mint_1 = accounts.token_mint_1.key();
    pool_state.token_vault_0 = accounts.token_vault_0.key();
    pool_state.token_vault_1 = accounts.token_vault_1.key();
    pool_state.observation_state = accounts.observation_state.key();
    pool_state.tick_array_bitmap = accounts.tick_array_bitmap.key();
    pool_state.mint_decimals_0 = accounts.token_mint_0.decimals;
    pool_state.mint_decimals_1 = accounts.token_mint_1.decimals;
    pool_state.tick_spacing = accounts.amm_config.tick_spacing;
    pool_state.tick_current = tick_current;
    pool_state.liquidity = 0;
    pool_state.sqrt_price_x64 = sqrt_price_x64;
    pool_state.fee_growth_global_0_x64 = 0;
    pool_state.fee_growth_global_1_x64 = 0;
    pool_state.protocol_fee_rate = accounts.amm_config.protocol_fee_rate;
    pool_state.fund_fee_rate = accounts.amm_config.fund_fee_rate;
    pool_state.trade_fee_rate = accounts.amm_config.trade_fee_rate;
    pool_state.protocol_fees_token_0 = 0;
    pool_state.protocol_fees_token_1 = 0;
    pool_state.status = 0;
    pool_state.padding0 = [0; 2];
    pool_state.reserved = [0; 32];

    accounts.pool_state.set_inner(pool_state);

    // initialize token vaults manually via CPI to support token-2022
    let token_program_0_info = accounts.token_program_0.to_account_info();
    let token_program_1_info = accounts.token_program_1.to_account_info();

    token::initialize_account3(
        CpiContext::new(
            token_program_0_info.clone(),
            token::InitializeAccount3 {
                account: accounts.token_vault_0.to_account_info(),
                mint: accounts.token_mint_0.to_account_info(),
                authority: accounts.pool_state.to_account_info(),
            },
        ),
    )?;

    token::initialize_account3(
        CpiContext::new(
            token_program_1_info,
            token::InitializeAccount3 {
                account: accounts.token_vault_1.to_account_info(),
                mint: accounts.token_mint_1.to_account_info(),
                authority: accounts.pool_state.to_account_info(),
            },
        ),
    )?;

    // initialize observation & tick array bitmap state
    accounts.observation_state.initialize(accounts.pool_state.key())?;
    accounts
        .tick_array_bitmap
        .initialize(accounts.pool_state.key());

    Ok(())
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

