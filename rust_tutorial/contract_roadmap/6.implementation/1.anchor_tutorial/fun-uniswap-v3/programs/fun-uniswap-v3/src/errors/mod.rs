use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Trade fee rate exceeds allowed maximum")]
    InvalidTradeFeeRate,
    #[msg("Protocol fee rate exceeds allowed maximum")]
    InvalidProtocolFeeRate,
    #[msg("Fund fee rate exceeds allowed maximum")]
    InvalidFundFeeRate,
    #[msg("Combined fee rates exceed 100% of trade volume")]
    TotalFeeRateTooHigh,
    #[msg("Fee tier / tick spacing must be greater than zero")]
    InvalidFeeTier,
    #[msg("AMM configuration account already initialized")]
    AlreadyInitialized,
    #[msg("Token mints must be supplied in ascending address order")]
    InvalidMintOrder,
    #[msg("Token decimal mismatch between the provided mints")]
    MintDecimalsMismatch,
    #[msg("Tick spacing does not match configuration requirements")]
    TickSpacingMismatch,
    #[msg("Pool state already initialized")]
    PoolAlreadyInitialized,
    #[msg("Provided signer does not match AMM configuration authority")]
    InvalidAuthority,
    #[msg("Missing PDA bump for derived account")]
    MissingBump,
}

