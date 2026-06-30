use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::MINT_SEED;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct TransferToken<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = sender,
        associated_token::token_program = token_program
    )]
    pub sender_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: allow any account to receive tokens
    pub recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = sender,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [MINT_SEED.as_bytes(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            token_interface::TransferChecked {
                from: ctx.accounts.sender_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.mint.decimals,
    )?;
    Ok(())
}
