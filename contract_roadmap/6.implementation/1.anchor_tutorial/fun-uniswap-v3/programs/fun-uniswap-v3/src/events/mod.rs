use anchor_lang::prelude::*;

#[event]
pub struct AmmConfigInitializedEvent {
    pub authority: Pubkey,
    pub index: u16,
    pub trade_fee_rate: u32,
    pub protocol_fee_rate: u32,
    pub fund_fee_rate: u32,
    pub tick_spacing: u16,
}