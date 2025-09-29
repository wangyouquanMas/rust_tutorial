use anchor_lang::prelude::*;

pub const TICK_ARRAY_BITMAP_SEED: &str = "tick-array-bitmap";
pub const TICK_ARRAY_BITMAP_EXTENSION_SEED: &str = "tick-array-bitmap-extension";

#[account]
#[derive(Debug, Default)]
pub struct TickArrayBitmapAccount {
    pub pool: Pubkey,
    pub bitmap: [u64; 16],
}

#[account]
#[derive(Debug, Default)]
pub struct TickArrayBitmapExtensionAccount {
    pub pool: Pubkey,
    pub positive_bitmap: [[u64; 8]; 14],
    pub negative_bitmap: [[u64; 8]; 14],
}

impl TickArrayBitmapExtensionAccount {
    pub const LEN: usize = 8 + 32 + 64 * 14 * 2;

    pub const fn space() -> usize {
        Self::LEN
    }

    pub fn initialize(&mut self, pool: Pubkey) {
        self.pool = pool;
        self.positive_bitmap = [[0; 8]; 14];
        self.negative_bitmap = [[0; 8]; 14];
    }
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

