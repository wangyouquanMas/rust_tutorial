use anchor_lang::prelude::*;

pub const TICK_ARRAY_BITMAP_SEED: &str = "tick-array-bitmap";

#[account]
#[derive(Debug, Default)]
pub struct TickArrayBitmapAccount {
    pub pool: Pubkey,
    pub bitmap: [u64; 16],
}

impl TickArrayBitmapAccount {
    pub const LEN: usize = 8 + 32 + (8 * 16);

    pub const fn space() -> usize {
        Self::LEN
    }

    pub fn initialize(&mut self, pool: Pubkey) {
        self.pool = pool;
        self.bitmap = [0; 16];
    }
}

