use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::libraries::{
    big_num::U512,
    tick_array_bit_map, tick_math,
};
use crate::state::tick_array::TickArrayState;

pub const TICK_ARRAY_BITMAP_SEED: &str = "tick-array-bitmap";
pub const TICK_ARRAY_BITMAP_EXTENSION_SEED: &str = "tick-array-bitmap-extension";
const EXTENSION_BITMAP_SIZE: usize = 14;

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
    pub positive_bitmap: [[u64; 8]; EXTENSION_BITMAP_SIZE],
    pub negative_bitmap: [[u64; 8]; EXTENSION_BITMAP_SIZE],
}

impl TickArrayBitmapExtensionAccount {
    pub const LEN: usize = 8 + 32 + 64 * 14 * 2;

    pub const fn space() -> usize {
        Self::LEN
    }

    pub fn initialize(&mut self, pool: Pubkey) {
        self.pool = pool;
        self.positive_bitmap = [[0; 8]; EXTENSION_BITMAP_SIZE];
        self.negative_bitmap = [[0; 8]; EXTENSION_BITMAP_SIZE];
    }

    fn get_bitmap_offset(tick_index: i32, tick_spacing: u16) -> Result<usize> {
        require!(
            TickArrayState::check_is_valid_start_index(tick_index, tick_spacing),
            ErrorCode::InvalidTickIndex
        );
        Self::check_extension_boundary(tick_index, tick_spacing)?;
        let ticks_in_bitmap = tick_array_bit_map::max_tick_in_tickarray_bitmap(tick_spacing);
        let mut offset = tick_index.abs() / ticks_in_bitmap - 1;
        if tick_index < 0 && tick_index.abs() % ticks_in_bitmap == 0 {
            offset -= 1;
        }
        Ok(offset as usize)
    }

    fn get_bitmap(&self, tick_index: i32, tick_spacing: u16) -> Result<(usize, [u64; 8])> {
        let offset = Self::get_bitmap_offset(tick_index, tick_spacing)?;
        if tick_index < 0 {
            Ok((offset, self.negative_bitmap[offset]))
        } else {
            Ok((offset, self.positive_bitmap[offset]))
        }
    }

    pub fn check_extension_boundary(tick_index: i32, tick_spacing: u16) -> Result<()> {
        let positive_tick_boundary = tick_array_bit_map::max_tick_in_tickarray_bitmap(tick_spacing);
        let negative_tick_boundary = -positive_tick_boundary;
        require_gt!(tick_math::MAX_TICK, positive_tick_boundary);
        require_gt!(negative_tick_boundary, tick_math::MIN_TICK);
        if tick_index >= negative_tick_boundary && tick_index < positive_tick_boundary {
            return err!(ErrorCode::InvalidTickArrayBoundary);
        }
        Ok(())
    }

    pub fn flip_tick_array_bit(
        &mut self,
        tick_array_start_index: i32,
        tick_spacing: u16,
    ) -> Result<()> {
        let (offset, bitmap) = self.get_bitmap(tick_array_start_index, tick_spacing)?;
        let tick_array_offset = Self::tick_array_offset_in_bitmap(tick_array_start_index, tick_spacing);
        let bitmap = U512(bitmap);
        let mask = U512::one() << tick_array_offset;
        if tick_array_start_index < 0 {
            self.negative_bitmap[offset] = bitmap.bitxor(mask).0;
        } else {
            self.positive_bitmap[offset] = bitmap.bitxor(mask).0;
        }
        Ok(())
    }

    pub fn check_tick_array_is_initialized(
        &self,
        tick_array_start_index: i32,
        tick_spacing: u16,
    ) -> Result<(bool, i32)> {
        let (_, bitmap) = self.get_bitmap(tick_array_start_index, tick_spacing)?;
        let tick_array_offset = Self::tick_array_offset_in_bitmap(tick_array_start_index, tick_spacing);
        if U512(bitmap).bit(tick_array_offset as usize) {
            Ok((true, tick_array_start_index))
        } else {
            Ok((false, tick_array_start_index))
        }
    }

    pub fn next_initialized_tick_array_from_one_bitmap(
        &self,
        last_tick_array_start_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> Result<(bool, i32)> {
        let multiplier = TickArrayState::tick_count(tick_spacing);
        let next_tick_array_start_index = if zero_for_one {
            last_tick_array_start_index - multiplier
        } else {
            last_tick_array_start_index + multiplier
        };

        let (_, bitmap) = self.get_bitmap(next_tick_array_start_index, tick_spacing)?;
        Ok(tick_array_bit_map::next_initialized_tick_array_in_bitmap(
            bitmap,
            next_tick_array_start_index,
            tick_spacing,
            zero_for_one,
        ))
    }

    pub fn tick_array_offset_in_bitmap(tick_array_start_index: i32, tick_spacing: u16) -> i32 {
        let max_tick = tick_array_bit_map::max_tick_in_tickarray_bitmap(tick_spacing);
        let m = tick_array_start_index.abs() % max_tick;
        let mut offset = m / TickArrayState::tick_count(tick_spacing);
        if tick_array_start_index < 0 && m != 0 {
            offset = tick_array_bit_map::TICK_ARRAY_BITMAP_SIZE - offset;
        }
        offset
    }
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

