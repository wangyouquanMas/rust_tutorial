# Step 2: Adding Market Impact Parameters Configuration

Let's implement the missing market impact parameters to control how trades affect pricing in the bonding curve:

## 1. First, Define New Fields in BondingCurve Struct

We need to update the `BondingCurve` struct in `state/bondingcurve.rs` to include market impact parameters:

```rust
#[account]
#[derive(Default)]
pub struct BondingCurve {
    pub token_mint: Pubkey,
    pub creator: Pubkey,
    pub init_lamport: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub token_total_supply: u64,
    pub is_completed: bool,
    
    // New market impact parameters
    pub slippage_bps_per_million_sol: u16,    // Base points of slippage per 1M SOL trade
    pub max_slippage_percent: u8,             // Maximum allowed slippage percentage
    pub min_trade_size_lamports: u64,         // Minimum trade size in lamports
    pub max_single_trade_sol: u64,            // Maximum single trade in SOL
    pub last_price_update_slot: u64,          // Slot of last price update
    pub average_price_lamports: u64,          // Average price in lamports for volatility calculation
    pub price_stability_factor: u8,           // 1-100 scale, higher means less volatile
}
```

## 2. Add Initialization Code to CreateBondingCurve

Now, integrate market impact parameter initialization to your `create_bonding_curve.rs` file after setting up the virtual reserves:

```rust
// After setting up virtual and real reserves...
self.bonding_curve.virtual_sol_reserves = self.global_config.initial_virtual_sol_reserves_config;
self.bonding_curve.virtual_token_reserves = self.global_config.initial_virtual_token_reserves_config;
self.bonding_curve.real_sol_reserves = 0;
self.bonding_curve.real_token_reserves = self.global_config.initial_real_token_reserves_config;
self.bonding_curve.token_total_supply = token_supply;

// Market impact parameter configuration
msg!("🔧 Configuring market impact parameters...");

// Calculate initial price in lamports per token (with precision handling)
let initial_price = ((self.bonding_curve.virtual_sol_reserves as u128) * 1_000_000) / 
                    (self.bonding_curve.virtual_token_reserves as u128);

// Set market impact parameters based on token supply and initial price
self.bonding_curve.slippage_bps_per_million_sol = 500;  // 5% slippage per 1M SOL trade
self.bonding_curve.max_slippage_percent = 25;           // 25% maximum slippage
self.bonding_curve.min_trade_size_lamports = 1_000_000; // 0.001 SOL minimum
self.bonding_curve.max_single_trade_sol = reserve_lamport * 10; // 10x initial reserve cap

// Price stability tracking
self.bonding_curve.last_price_update_slot = Clock::get()?.slot;
self.bonding_curve.average_price_lamports = initial_price as u64;
self.bonding_curve.price_stability_factor = 50; // Medium stability to start

msg!("✅ Market impact parameters configured");

// Initial price analytics
msg!("📊 Initial market parameters:");
msg!("  - Initial price (lamports/token): {}", initial_price);
msg!("  - Slippage rate: {} bps per 1M SOL", self.bonding_curve.slippage_bps_per_million_sol);
msg!("  - Max slippage: {}%", self.bonding_curve.max_slippage_percent);
msg!("  - Min trade: {} lamports", self.bonding_curve.min_trade_size_lamports);
msg!("  - Max trade: {} SOL", self.bonding_curve.max_single_trade_sol / 1_000_000_000);
```

## 3. Add Import for Clock

Add this import at the top of your file:

```rust
use solana_program::clock::Clock;
```

## 4. Add Error Handling for Clock

Add a new error type in your `errors.rs` file if it doesn't exist:

```rust
#[error_code]
pub enum ContractError {
    // ... existing errors
    
    #[msg("Failed to get clock")]
    ClockUnavailable,
}
```

## 5. Integration with Swap Function

These market impact parameters will be used in your swap function to:
1. Calculate price impact based on trade size
2. Apply slippage limitations
3. Prevent excessively large trades
4. Track price stability over time

## Explanation of Parameters:

- **slippage_bps_per_million_sol**: Controls how much price slippage a 1M SOL trade would cause
- **max_slippage_percent**: Hard cap on maximum allowed slippage
- **min_trade_size_lamports**: Prevents dust trades that waste resources
- **max_single_trade_sol**: Prevents market manipulation through massive trades
- **price_stability_factor**: Can be used to adjust curve parameters over time based on volatility
- **average_price_lamports**: Tracks average price for monitoring volatility
- **last_price_update_slot**: Tracks when price was last updated

These parameters allow you to create a smoother trading experience with controlled price impact, making the bonding curve more resistant to manipulation and providing better UX for traders.

Would you like to move on to the next step of our implementation?