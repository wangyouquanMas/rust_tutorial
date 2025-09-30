# Step 3: Implementing Initial Token Distribution Logic

Let's implement the initial token distribution to give some tokens to the creator/team when creating a bonding curve:

## 1. Define Distribution Parameters in Config

First, we need to add a parameter in the `Config` struct to define the percentage of tokens that go to creator/team. If it's not already there, add this to your `state/config.rs` file:

```rust
#[account]
#[derive(Debug)]
pub struct Config {
    // Existing fields...
    
    pub creator_token_allocation_percentage: u8, // 0-100%, how much goes to creator
    
    // Rest of existing fields...
}
```

## 2. Add Distribution Logic to CreateBondingCurve

Insert this code after minting the total supply but before revoking the mint authority:

```rust
// After minting tokens to global token account but before revoking mint authority

// Calculate creator allocation
let creator_allocation_percentage = self.global_config.creator_token_allocation_percentage as u64;
if creator_allocation_percentage > 0 {
    msg!("🏗️ Setting up creator token distribution...");
    
    // Calculate token amount for creator (percentage of total supply)
    let creator_token_amount = (token_supply * creator_allocation_percentage) / 100;
    
    // Check if creator already has a token account
    let creator_ata = anchor_spl::associated_token::get_associated_token_address(
        &creator_key,
        &token_key
    );
    
    let creator_ata_info = self.token_program.to_account_info().clone();
    let creator_ata_exists = creator_ata_info.data_len() > 0;
    
    // Create creator token account if it doesn't exist
    if !creator_ata_exists {
        msg!("📝 Creating creator token account...");
        associated_token::create(CpiContext::new(
            self.associated_token_program.to_account_info(),
            associated_token::Create {
                payer: self.creator.to_account_info(),
                associated_token: self.creator.to_account_info(),
                authority: self.creator.to_account_info(),
                mint: self.token.to_account_info(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        ))?;
        msg!("✅ Creator token account created");
    }
    
    // Transfer tokens to creator
    msg!("💸 Transferring {} tokens to creator...", creator_token_amount);
    token::transfer(
        CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.global_token_account.to_account_info(),
                to: AccountInfo::new(
                    creator_ata,
                    false,
                    false,
                    &self.token.to_account_info().data_len(),
                    &self.token.to_account_info().data,
                    &self.token.to_account_info().owner,
                    self.token.to_account_info().executable,
                    self.token.to_account_info().rent_epoch,
                ),
                authority: self.global_vault.to_account_info(),
            },
            signer_seeds,
        ),
        creator_token_amount,
    )?;
    msg!("✅ Creator token allocation complete");
    
    // Update real token reserves (subtract creator allocation)
    self.bonding_curve.real_token_reserves -= creator_token_amount;
    msg!("📊 Updated real token reserves: {}", self.bonding_curve.real_token_reserves);
}

// Team wallet allocation (optional)
let team_allocation_percentage = 2; // Example: 2% goes to team wallet
if team_allocation_percentage > 0 {
    msg!("🏗️ Setting up team wallet token distribution...");
    
    // Calculate token amount for team (percentage of total supply)
    let team_token_amount = (token_supply * team_allocation_percentage) / 100;
    
    // Get team wallet ATA
    let team_wallet_key = self.team_wallet.key();
    let team_wallet_ata = anchor_spl::associated_token::get_associated_token_address(
        &team_wallet_key,
        &token_key
    );
    
    // Create team wallet token account
    msg!("📝 Creating team wallet token account...");
    associated_token::create(CpiContext::new(
        self.associated_token_program.to_account_info(),
        associated_token::Create {
            payer: self.creator.to_account_info(),
            associated_token: AccountInfo::new(
                team_wallet_ata,
                false,
                false,
                &self.token.to_account_info().data_len(),
                &self.token.to_account_info().data,
                &self.token.to_account_info().owner,
                self.token.to_account_info().executable,
                self.token.to_account_info().rent_epoch,
            ),
            authority: self.team_wallet.to_account_info(),
            mint: self.token.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        },
    ))?;
    
    // Transfer tokens to team wallet
    msg!("💸 Transferring {} tokens to team wallet...", team_token_amount);
    token::transfer(
        CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.global_token_account.to_account_info(),
                to: AccountInfo::new(
                    team_wallet_ata,
                    false,
                    false,
                    &self.token.to_account_info().data_len(),
                    &self.token.to_account_info().data,
                    &self.token.to_account_info().owner,
                    self.token.to_account_info().executable,
                    self.token.to_account_info().rent_epoch,
                ),
                authority: self.global_vault.to_account_info(),
            },
            signer_seeds,
        ),
        team_token_amount,
    )?;
    msg!("✅ Team wallet token allocation complete");
    
    // Update real token reserves (subtract team allocation)
    self.bonding_curve.real_token_reserves -= team_token_amount;
    msg!("📊 Updated real token reserves: {}", self.bonding_curve.real_token_reserves);
}
```

## 3. Add Initialization Method in Config (If Needed)

If the `creator_token_allocation_percentage` field is new, make sure it's properly initialized in the Configure instruction. Update the `configure.rs` file to include:

```rust
// In the configure instruction handler:
new_config.creator_token_allocation_percentage = new_config.creator_token_allocation_percentage.min(20); // Cap at 20%
```

## 4. Track Distributions in BondingCurve

Add fields to track these distributions in the `BondingCurve` struct:

```rust
#[account]
#[derive(Default)]
pub struct BondingCurve {
    // Existing fields...
    
    pub creator_allocation: u64,    // Amount allocated to creator
    pub team_allocation: u64,       // Amount allocated to team wallet
    
    // Other existing fields...
}
```

And update them in your distribution logic:

```rust
// After transferring to creator:
self.bonding_curve.creator_allocation = creator_token_amount;

// After transferring to team wallet:
self.bonding_curve.team_allocation = team_token_amount;
```

## 5. Add Event for Token Distribution

Create an event to track the initial distribution:

```rust
// In events.rs:
#[event]
pub struct TokenDistributionEvent {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub team_wallet: Pubkey,
    pub creator_amount: u64,
    pub team_amount: u64,
    pub remaining_supply: u64,
}

// In your code after both distributions:
emit!(TokenDistributionEvent {
    mint: token_key,
    creator: creator_key,
    team_wallet: self.team_wallet.key(),
    creator_amount: creator_token_amount,
    team_amount: team_token_amount,
    remaining_supply: self.bonding_curve.real_token_reserves,
});
```

This implementation provides flexible distribution logic that:
1. Gives a configurable percentage to the creator
2. Gives a fixed percentage to the team wallet
3. Updates accounting in the bonding curve
4. Emits events for tracking
5. Ensures token accounts are properly created

Would you like to proceed with implementing this, or would you like to adjust any part of the distribution logic?