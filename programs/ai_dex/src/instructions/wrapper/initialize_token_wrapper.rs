use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[event]
pub struct TokenWrapperInitializedEvent {
    pub ai_dex_config: Pubkey,
    pub token_wrapper_authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_wrapper: Pubkey,
    pub funder: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeTokenWrapper<'info> {
    pub ai_dex_config: Box<Account<'info, AiDexConfig>>,

    #[account(address = ai_dex_config.config_authority)]
    pub token_wrapper_authority: Signer<'info>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = funder,
        seeds = [
            b"token_wrapper",
            ai_dex_config.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        space = TokenWrapper::LEN
    )]
    pub token_wrapper: Account<'info, TokenWrapper>,

    #[account(mut)]
    pub funder: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Initializes a token wrapper in the protocol.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts and programs required for the operation.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the operation is successful, otherwise returns an error.
///
/// # Errors
///
/// * Any error that occurs during the initialization of the token wrapper.
pub fn initialize_token_wrapper_handler(
    ctx: Context<InitializeTokenWrapper>,
) -> Result<()> {
    ctx
        .accounts
        .token_wrapper
        .initialize(
            ctx.accounts.ai_dex_config.key(),
            ctx.accounts.token_mint.key(),
        )?;
        
        emit!(TokenWrapperInitializedEvent {
            ai_dex_config: ctx.accounts.ai_dex_config.key(),
            token_wrapper_authority: ctx.accounts.token_wrapper_authority.key(),
            token_mint: ctx.accounts.token_mint.key(),
            token_wrapper: ctx.accounts.token_wrapper.key(),
            funder: ctx.accounts.funder.key(),
        });        

    Ok(())
}
