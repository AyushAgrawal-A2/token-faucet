pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2m78e8ia8SuG9jxnLFtVBmvi5TsL8zaMWq3FRENAbRzZ");

#[program]
pub mod token_faucet {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        _seed: u64,
        _decimals: u8,
        faucet_authority: Pubkey,
        max_supply: u64,
        mint_timeout: i64,
        mint_limit: u64,
    ) -> Result<()> {
        initialize::handler(ctx, faucet_authority, max_supply, mint_timeout, mint_limit)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        _seed: u64,
        max_supply: u64,
        mint_timeout: i64,
        mint_limit: u64,
    ) -> Result<()> {
        update_config::handler(ctx, max_supply, mint_timeout, mint_limit)
    }

    pub fn mint_token(ctx: Context<MintToken>, seed: u64, amount: u64) -> Result<()> {
        mint_token::handler(ctx, seed, amount)
    }

    pub fn transfer_token(ctx: Context<TransferToken>, _seed: u64, amount: u64) -> Result<()> {
        transfer_token::handler(ctx, amount)
    }
}
