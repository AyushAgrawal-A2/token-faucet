use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct FaucetConfig {
    pub admin: Pubkey,
    pub max_supply: u64,
    pub mint_timeout: i64,
    pub mint_limit: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct MintTimeout {
    pub amount_minted: u64,
    pub timeout_reset: i64,
}
