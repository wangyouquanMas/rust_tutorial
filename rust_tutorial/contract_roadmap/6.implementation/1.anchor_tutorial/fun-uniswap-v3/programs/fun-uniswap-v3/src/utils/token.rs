use std::collections::HashSet;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Token};
use anchor_spl::token_2022::{self, get_account_data_size, GetAccountDataSize, InitializeAccount3, InitializeImmutableOwner, Token2022};
use anchor_spl::token_interface::{self as token_interface, Mint, TokenAccount, TokenInterface};
use spl_token_2022::{self, extension::{default_account_state::DefaultAccountState, metadata_pointer, transfer_fee::{TransferFeeConfig, MAX_FEE_BASIS_POINTS}, BaseStateWithExtensions, ExtensionType, StateWithExtensions}, state::AccountState};

use crate::errors::ErrorCode;
use crate::state::support_mint_associated::SupportMintAssociated;
use crate::utils::system::create_or_allocate_account;

const MINT_WHITELIST: [&'static str; 0] = [];

pub fn create_token_vault_account<'info>(
    payer: &Signer<'info>,
    pool_state: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    token_mint: &InterfaceAccount<'info, Mint>,
    system_program: &Program<'info, System>,
    token_program: &Interface<'info, TokenInterface>,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    let immutable_owner_required = is_superstate_token(token_mint);
    let space = get_account_data_size(
        CpiContext::new(
            token_program.to_account_info(),
            GetAccountDataSize {
                mint: token_mint.to_account_info(),
            },
        ),
        if immutable_owner_required {
            &[ExtensionType::ImmutableOwner]
        } else {
            &[]
        },
    )?;

    create_or_allocate_account(
        token_program.key,
        payer.to_account_info(),
        system_program.to_account_info(),
        token_account.clone(),
        signer_seeds,
        space.try_into().unwrap(),
    )?;

    if immutable_owner_required {
        token_2022::initialize_immutable_owner(CpiContext::new(
            token_program.to_account_info(),
            InitializeImmutableOwner {
                account: token_account.clone(),
            },
        ))?;
    }

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

pub fn support_mint_associated_is_initialized(
    remaining_accounts: &[AccountInfo],
    token_mint: &InterfaceAccount<Mint>,
) -> Result<bool> {
    if remaining_accounts.is_empty() {
        return Ok(false);
    }
    let (expect_mint_associated, _bump) = Pubkey::find_program_address(
        &[SupportMintAssociated::SEED, token_mint.key().as_ref()],
        &crate::id(),
    );
    for info in remaining_accounts.iter() {
        if *info.owner != crate::id() || info.key() != expect_mint_associated {
            continue;
        }
        let mint_associated = SupportMintAssociated::try_deserialize(
            &mut info.data.borrow().as_ref(),
        )?;
        if mint_associated.mint == token_mint.key() {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn is_supported_mint(
    mint_account: &InterfaceAccount<Mint>,
    mint_associated_is_initialized: bool,
) -> Result<bool> {
    let mint_info = mint_account.to_account_info();
    if *mint_info.owner == Token::id() {
        return Ok(true);
    }
    let mint_whitelist: HashSet<&str> = MINT_WHITELIST.into_iter().collect();
    if mint_whitelist.contains(mint_account.key().to_string().as_str()) {
        return Ok(true);
    }
    if mint_associated_is_initialized {
        return Ok(true);
    }
    if is_superstate_token(mint_account) {
        return Ok(true);
    }
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;
    for e in mint.get_extension_types()? {
        if e != ExtensionType::TransferFeeConfig
            && e != ExtensionType::MetadataPointer
            && e != ExtensionType::TokenMetadata
            && e != ExtensionType::InterestBearingConfig
            && e != ExtensionType::ScaledUiAmount
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn is_superstate_token(mint_account: &InterfaceAccount<Mint>) -> bool {
    if let COption::Some(freeze_authority) = mint_account.freeze_authority {
        let mint_account_info = mint_account.to_account_info();
        let mint_data = mint_account_info.try_borrow_data().unwrap();
        let mint_state =
            StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data).unwrap();
        let default_account_state_freeze =
            if let Ok(default_account_state) = mint_state.get_extension::<DefaultAccountState>() {
                default_account_state.state == (AccountState::Frozen as u8)
            } else {
                false
            };

        default_account_state_freeze
    } else {
        false
    }
}

