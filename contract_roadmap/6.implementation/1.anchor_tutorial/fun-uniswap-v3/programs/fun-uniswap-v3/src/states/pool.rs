pub const POOL_SEED: &str = "pool";
pub const POOL_TICK_ARRAY_BITMAP_SEED: &str = "pool_tick_array_bitmap_extension";


// #[account(zero_copy(unsafe))]
// #[repr(C, packed)]
// #[derive(Default, Debug)]
// pub struct PoolState {
//     pub bump: u8,
//     pub amm_config: Pubkey,
//     //The creator of the pool
//     pub owner: Pubkey,

//     //Two tokens 
//     pub token_mint_0: Pubkey,
//     pub token_mint_1: Pubkey,

//     //Two token vaults
//     pub token_vault_0: Pubkey,
//     pub token_vault_1: Pubkey,

//     //Mint0 and Mint1 decimals 
//     pub mint_decimals_0: u8,
//     pub mint_decimals_1: u8,

//     //The minimun number of ticks between initialized ticks
//     pub tick_spacing: u16,
//     //The currently in range liquidity available to thep ool
//     pub liquidity: u128,
//     //The current price of the pool as a sqrt Q64.64. value 
//     pub sqrt_price_x64: u128,
//     //The current tick of the pool. 
//     pub tick_current: i32,
// }

// impl PoolState{
//     pub const LEN: usize = 8+
//     +1
//     +32 * 6
//     +1
//     +1
//     +2
//     +16
//     +16
//     +4;

//     pub fn seeds(&self) -> [&[u8]; 5]{
//         [
//             &POOL_SEED.as_bytes(),
//             self.amm_config.as_ref(),
//             self.token_mint_0.as_ref(),
//             self.token_mint_1.as_ref(),
//             self.bump.as_ref(),
//         ]
//     }

//     pub fn initialize(
//         &mut self,
//         bump: u8,
//         sqrt_price_x64: u128,
//         tick: i32,
//         pool_creator: PubKey,
//         token_vault_0: Pubkey,
//         token_vault_1: Pubkey,
//         amm_config: &Account<AmmConfig>,
//         token_mint_0: &InterfaceAccount<Mint>,
//         token_mint_1: &InterfaceAccount<Mint>,
//     ) -> Result<()> {
//         self.bump = [bump];
//         self.amm_config = amm_config.key();
//         self.owner = pool_creator.key();
//         self.token_mint_0 = token_mint_0.key();
//         self.token_mint_1 = token_mint_1.key();
//         self.mint_decimals_0 = token_mint_0.decimals;
//         self.mint_decimals_1 = token_mint_1.decimals;
//         self.token_vault_0 = token_vault_0;
//         self.token_vault_1 = token_vault_1;
//         self.tick_spacing = amm_config.tick_spacing;
//         self.liquidity = 0;
//         self.sqrt_price_x64 = sqrt_price_x64;
//         self.tick_current = tick;
//         Ok(())
//     }

// }