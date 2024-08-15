use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[event]
pub struct TokenWrapperDeletedEvent {
    pub ai_dex_config: Pubkey,
    pub token_wrapper_authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_wrapper: Pubkey,
    pub receiver: Pubkey,
}

#[derive(Accounts)]
pub struct DeleteTokenWrapper<'info> {
    pub ai_dex_config: Box<Account<'info, AiDexConfig>>,

    #[account(address = ai_dex_config.config_authority)]
    pub token_wrapper_authority: Signer<'info>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [
            b"token_wrapper",
            ai_dex_config.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        has_one = ai_dex_config,
        close = receiver
    )]
    pub token_wrapper: Account<'info, TokenWrapper>,

    /// CHECK: safe, for receiving rent only
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,
}

/// Handles the deletion of a token wrapper in the protocol.
///
/// This function ensures that the token wrapper is properly closed and any remaining rent is transferred to the receiver account.
///
/// # Arguments
///
/// * `_ctx` - The context containing all the accounts required for the token wrapper deletion.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the token wrapper deletion is successful, otherwise returns an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The token wrapper account cannot be closed.
/// * The rent cannot be transferred to the receiver account.
pub fn delete_token_wrapper_handler(
    ctx: Context<DeleteTokenWrapper>,
) -> Result<()> {
    // The account closure happens automatically due to the `close = receiver` constraint in the `Accounts` struct.

    emit!(TokenWrapperDeletedEvent {
        ai_dex_config: ctx.accounts.ai_dex_config.key(),
        token_wrapper_authority: ctx.accounts.token_wrapper_authority.key(),
        token_mint: ctx.accounts.token_mint.key(),
        token_wrapper: ctx.accounts.token_wrapper.key(),
        receiver: ctx.accounts.receiver.key(),
    });
    
    Ok(())
}
