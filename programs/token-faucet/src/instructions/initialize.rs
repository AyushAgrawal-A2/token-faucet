use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{error::TokenFaucetError, FaucetConfig, FAUCET_SEED, MINT_SEED};

#[derive(Accounts)]
#[instruction(seed: u64, decimals: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + FaucetConfig::INIT_SPACE,
        seeds = [FAUCET_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub faucet_config: Account<'info, FaucetConfig>,

    #[account(
        init,
        payer = admin,
        mint::decimals = decimals,
        mint::authority = mint,
        seeds = [MINT_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(
    ctx: Context<Initialize>,
    max_supply: u64,
    mint_timeout: i64,
    mint_limit: u64,
) -> Result<()> {
    require!(mint_timeout >= 0, TokenFaucetError::InvalidMintTimeout);
    ctx.accounts.faucet_config.admin = ctx.accounts.admin.key();
    ctx.accounts.faucet_config.max_supply = max_supply;
    ctx.accounts.faucet_config.mint_timeout = mint_timeout;
    ctx.accounts.faucet_config.mint_limit = mint_limit;
    ctx.accounts.faucet_config.bump = ctx.bumps.faucet_config;
    Ok(())
}
