use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_interface::{self as token_interface, Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;

pub fn create_token_vault_account<'info>(
    payer: &Signer<'info>,
    pool_state: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    token_mint: &InterfaceAccount<'info, Mint>,
    system_program: &Program<'info, System>,
    token_program: &Interface<'info, TokenInterface>,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    let rent = Rent::get()?;
    let space = TokenAccount::LEN;
    let lamports = rent.minimum_balance(space);

    let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
        payer.key,
        token_account.key,
        lamports,
        space as u64,
        token_program.key,
    );

    invoke_signed(
        &create_account_ix,
        &[payer.to_account_info(), token_account.clone(), system_program.to_account_info()],
        &[signer_seeds],
    )?;

    token_interface::initialize_account3(
        CpiContext::new(
            token_program.to_account_info(),
            token_interface::InitializeAccount3 {
                account: token_account.clone(),
                mint: token_mint.to_account_info(),
                authority: pool_state.clone(),
            },
        ),
    )
    .map_err(|_| ErrorCode::VaultBumpMismatch.into())
}

