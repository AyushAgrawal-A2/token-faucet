use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{
    error::TokenFaucetError, FaucetConfig, MintTimeout, FAUCET_SEED, MINT_SEED, MINT_TIMEOUT_SEED,
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program
    )]
    pub token_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [FAUCET_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump = faucet_config.bump
    )]
    pub faucet_config: Account<'info, FaucetConfig>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + MintTimeout::INIT_SPACE,
        seeds = [MINT_TIMEOUT_SEED.as_bytes(), seed.to_le_bytes().as_ref(), payer.key().as_ref()],
        bump
    )]
    pub mint_timeout: Account<'info, MintTimeout>,

    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<MintToken>, seed: u64, amount: u64) -> Result<()> {
    let current_timestamp = Clock::get()?.unix_timestamp;
    if ctx.accounts.mint_timeout.timeout_reset <= current_timestamp {
        ctx.accounts.mint_timeout.amount_minted = amount;
        ctx.accounts.mint_timeout.timeout_reset = current_timestamp
            .checked_add(ctx.accounts.faucet_config.mint_timeout)
            .ok_or(TokenFaucetError::Overflow)?;
    } else {
        ctx.accounts.mint_timeout.amount_minted = ctx
            .accounts
            .mint_timeout
            .amount_minted
            .checked_add(amount)
            .ok_or(TokenFaucetError::Overflow)?;
    }
    require!(
        ctx.accounts.mint_timeout.amount_minted <= ctx.accounts.faucet_config.mint_limit,
        TokenFaucetError::MintTimeoutExceeded
    );

    let new_supply = ctx
        .accounts
        .mint
        .supply
        .checked_add(amount)
        .ok_or(TokenFaucetError::Overflow)?;
    require!(
        new_supply <= ctx.accounts.faucet_config.max_supply,
        TokenFaucetError::MintExceedsMaxSupply
    );

    let seed = seed.to_le_bytes();
    let bump = [ctx.bumps.mint];
    let signer_seeds: &[&[&[u8]]] = &[&[MINT_SEED.as_bytes(), seed.as_ref(), &bump][..]];
    token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_ata.to_account_info(),
                authority: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;
    Ok(())
}
