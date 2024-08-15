use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    errors::ErrorCode,
    state::AiDexPool,
    util::{is_token_wrapper_initialized, is_supported_token_mint}
};

#[event]
pub struct RewardInitializedEvent {
    pub reward_index: u8,
    pub ai_dex: Pubkey,
    pub reward_authority: Pubkey,
    pub funder: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_token_wrapper: Pubkey,
    pub reward_vault: Pubkey,
    pub is_token_wrapper_initialized: bool,
}

#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct InitializeReward<'info> {
    #[account(address = ai_dex_pool.reward_infos[reward_index as usize].authority)]
    pub reward_authority: Signer<'info>,

    #[account(mut)]
    pub funder: Signer<'info>,

    #[account(mut)]
    pub ai_dex_pool: Box<Account<'info, AiDexPool>>,

    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: checked in the handler
    #[account(
        seeds = [
            b"token_wrapper",
            ai_dex_pool.ai_dex_config.as_ref(),
            reward_mint.key().as_ref()],
            bump,
    )]
    pub reward_token_wrapper: UncheckedAccount<'info>,

    #[account(
        init,
        payer = funder,
        token::token_program = reward_token_program,
        token::mint = reward_mint,
        token::authority = ai_dex_pool
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = reward_mint.to_account_info().owner.clone())]
    pub reward_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Initializes a reward in the protocol.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts and programs required for the operation.
/// * `reward_index` - The index of the reward to be initialized.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the operation is successful, otherwise returns an error.
///
/// # Errors
///
/// * `ErrorCode::UnsupportedTokenMintError` - If the token mint is not supported.
pub fn initialize_reward_handler(ctx: Context<InitializeReward>, reward_index: u8) -> Result<()> {
    let ai_dex = &mut ctx.accounts.ai_dex_pool;

    // Don't allow initializing a reward with an unsupported token mint
    let is_token_wrapper_initialized = is_token_wrapper_initialized(
        ai_dex.ai_dex_config,
        ctx.accounts.reward_mint.key(),
        &ctx.accounts.reward_token_wrapper,
    )?;
  
    if !is_supported_token_mint(&ctx.accounts.reward_mint, is_token_wrapper_initialized).unwrap() {
        return Err(ErrorCode::UnsupportedTokenMintError.into());
    }  

    ai_dex.initialize_reward(
        reward_index as usize,
        ctx.accounts.reward_mint.key(),
        ctx.accounts.reward_vault.key(),
    )?;

    emit!(RewardInitializedEvent {
        reward_index,
        ai_dex: ctx.accounts.ai_dex_pool.key(),
        reward_authority: ctx.accounts.reward_authority.key(),
        funder: ctx.accounts.funder.key(),
        reward_mint: ctx.accounts.reward_mint.key(),
        reward_token_wrapper: ctx.accounts.reward_token_wrapper.key(),
        reward_vault: ctx.accounts.reward_vault.key(),
        is_token_wrapper_initialized,
    });
    
    Ok(())
}
