use crate::libraries::tick_math;

pub const TICK_ARRAY_SEED: &str = "tick_array";
pub const TICK_ARRAY_SIZE_USIZE: usize = 60;
pub const TICK_ARRAY_SIZE: i32 = 60;

pub struct TickArrayState;

impl TickArrayState {
    pub fn tick_count(tick_spacing: u16) -> i32 {
        TICK_ARRAY_SIZE * i32::from(tick_spacing)
    }

    pub fn check_is_out_of_boundary(tick: i32) -> bool {
        tick < tick_math::MIN_TICK || tick > tick_math::MAX_TICK
    }

    pub fn check_is_valid_start_index(start_tick: i32, tick_spacing: u16) -> bool {
        if Self::check_is_out_of_boundary(start_tick) {
            return false;
        }
        start_tick % Self::tick_count(tick_spacing) == 0
    }
}

