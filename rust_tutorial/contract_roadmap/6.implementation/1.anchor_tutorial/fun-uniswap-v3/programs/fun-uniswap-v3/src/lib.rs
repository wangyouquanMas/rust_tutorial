use anchor_lang::prelude::*;
pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;
pub mod utils;

declare_id!("HXdTUsTLztg64qJ8upPqHMnBAmeLA8Zmj1xUhDPqj4fn");

#[program]
pub mod fun_uniswap_v3 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
