# How to Integrate Enhanced Validation into Your Codebase

Here's exactly how to integrate the enhanced validation code into your existing `create_bonding_curve.rs` file:

## 1. Locate the Integration Point

Find this section in your code (around line 110):

```rust
msg!("🔍 Validating decimals config...");
self.global_config.token_decimals_config.validate(&decimals)?;
msg!("✅ Decimals config validation passed");

// create token launch pda
msg!("🏗️  Setting up bonding curve...");
```

## 2. Insert New Validation Code

Add the new validation code between these sections:

```rust
msg!("🔍 Validating decimals config...");
self.global_config.token_decimals_config.validate(&decimals)?;
msg!("✅ Decimals config validation passed");

// INSERT NEW CODE HERE - BEGIN
// ENHANCED VALIDATION SECTION
msg!("🔍 Performing extended validation...");

// 1. Name and symbol validation
if name.len() > 32 {
    msg!("❌ Token name too long (max 32 chars)");
    return Err(ValueInvalid.into());
}

if symbol.len() > 10 {
    msg!("❌ Token symbol too long (max 10 chars)");
    return Err(ValueInvalid.into());
}

if symbol.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
    msg!("❌ Token symbol contains invalid characters");
    return Err(ValueInvalid.into());
}

// 2. URI validation
if uri.len() > 200 {
    msg!("❌ Token URI too long (max 200 chars)");
    return Err(ValueInvalid.into());
}

if !uri.starts_with("http") && !uri.starts_with("ipfs://") && !uri.starts_with("ar://") {
    msg!("❌ Token URI must start with http, ipfs:// or ar://");
    return Err(ValueInvalid.into());
}

// 3. Reserve validation beyond config
if reserve_lamport < 10_000_000 { // 0.01 SOL minimum
    msg!("❌ Reserve lamport amount too small (min 0.01 SOL)");
    return Err(ValueTooSmall.into());
}

// 4. Check for reasonable token supply
if token_supply < 1_000_000 * decimal_multiplier {
    msg!("❌ Token supply too small (min 1M tokens)");
    return Err(ValueTooSmall.into());
}

// 5. Virtual reserves validation
let virtual_sol_reserves = self.global_config.initial_virtual_sol_reserves_config;
let virtual_token_reserves = self.global_config.initial_virtual_token_reserves_config;

if virtual_sol_reserves == 0 || virtual_token_reserves == 0 {
    msg!("❌ Virtual reserves cannot be zero");
    return Err(ValueInvalid.into());
}

// 6. Check initial price is reasonable
let initial_price_lamports = (virtual_sol_reserves as f64) / (virtual_token_reserves as f64);
let min_reasonable_price = 0.000000001; // 1 lamport per 1000 tokens
let max_reasonable_price = 0.001; // 1000 lamports per token

if initial_price_lamports < min_reasonable_price || initial_price_lamports > max_reasonable_price {
    msg!("❌ Initial price is outside reasonable bounds");
    msg!("   Price: {} lamports per token", initial_price_lamports);
    return Err(ValueInvalid.into());
}

msg!("✅ Extended validation passed");
// INSERT NEW CODE HERE - END

// create token launch pda
msg!("🏗️  Setting up bonding curve...");
```

## 3. Add Any Missing Errors

If your `errors.rs` file doesn't already have the error types we're using, add these:

```rust
// In errors.rs (if not already present)
#[error_code]
pub enum ContractError {
    // ... existing errors ...
    
    #[msg("Value is too small")]
    ValueTooSmall,
    
    #[msg("Value is too large")]
    ValueTooLarge,
    
    #[msg("Value is invalid")]
    ValueInvalid,
}
```

## 4. Build and Test

After integrating the code:

```bash
RUSTUP_TOOLCHAIN="nightly-2024-11-19" anchor build
```

Test the validation with both valid and invalid inputs to ensure it works correctly.

## Notes:

1. The price calculation uses floating point math which is generally not ideal for financial calculations, but since this is only for validation (not actual token math), it's acceptable.

2. You might want to adjust the specific thresholds based on your exact requirements:
   - Minimum SOL reserve (currently 0.01 SOL)
   - Minimum token supply (currently 1M tokens)
   - Price boundaries (currently 0.000000001 to 0.001 SOL per token)

3. If you already have `ValueTooSmall`, `ValueTooLarge`, and `ValueInvalid` error types, you don't need to add them again.

Would you like to proceed to implementing the next step in completing the bonding curve logic?