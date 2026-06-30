use anchor_lang::prelude::*;

#[error_code]
pub enum TokenFaucetError {
    #[msg("Overflow")]
    Overflow,
    #[msg("Mint exceed max supply")]
    MintExceedsMaxSupply,
    #[msg("Max supply needs to be larger than minted tokens")]
    InvalidMaxSupply,
    #[msg("Exceeded mint limit, wait for timeout")]
    MintTimeoutExceeded,
}
