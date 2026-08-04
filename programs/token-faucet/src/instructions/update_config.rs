use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{error::TokenFaucetError, FaucetConfig, FAUCET_SEED, MINT_SEED};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin @ TokenFaucetError::Unauthorized,
        seeds = [FAUCET_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump = faucet_config.bump,
    )]
    pub faucet_config: Account<'info, FaucetConfig>,

    #[account(
        seeds = [MINT_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
}

pub fn handler(
    ctx: Context<UpdateConfig>,
    max_supply: u64,
    mint_timeout: i64,
    mint_limit: u64,
) -> Result<()> {
    require!(mint_timeout >= 0, TokenFaucetError::InvalidMintTimeout);
    require!(
        max_supply >= ctx.accounts.mint.supply,
        TokenFaucetError::InvalidMaxSupply
    );
    ctx.accounts.faucet_config.max_supply = max_supply;
    ctx.accounts.faucet_config.mint_timeout = mint_timeout;
    ctx.accounts.faucet_config.mint_limit = mint_limit;
    Ok(())
}
