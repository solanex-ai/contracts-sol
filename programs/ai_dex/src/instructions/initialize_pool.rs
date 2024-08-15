use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
  errors::ErrorCode,
  state::*,
  util::{is_token_wrapper_initialized, is_supported_token_mint}
};

#[event]
pub struct PoolInitializedEvent {
    pub ai_dex_pool: Pubkey,
    pub ai_dex_config: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_wrapper_a: Pubkey,
    pub token_wrapper_b: Pubkey,
    pub funder: Pubkey,
    pub tick_spacing: u16,
    pub initial_sqrt_price: u128,
    pub default_fee_rate: u16,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_tier: Pubkey,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
}

#[derive(Accounts)]
#[instruction(tick_spacing: u16)]
pub struct InitializePool<'info> {
    pub ai_dex_config: Box<Account<'info, AiDexConfig>>,

    pub token_mint_a: InterfaceAccount<'info, Mint>,
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    /// CHECK: checked in the handler
    #[account(
        seeds = [
            b"token_wrapper",
            ai_dex_config.key().as_ref(),
            token_mint_a.key().as_ref()
        ],
        bump
    )]
    pub token_wrapper_a: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"token_wrapper",
            ai_dex_config.key().as_ref(),
            token_mint_b.key().as_ref()
            ],
        bump
    )]
    /// CHECK: checked in the handler
    pub token_wrapper_b: UncheckedAccount<'info>,

    #[account(mut)]
    pub funder: Signer<'info>,

    #[account(
        init,
        seeds = [
            b"ai_dex".as_ref(),
            ai_dex_config.key().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
            tick_spacing.to_le_bytes().as_ref()
        ],
        bump,
        payer = funder,
        space = AiDexPool::LEN
    )]
    pub ai_dex_pool: Box<Account<'info, AiDexPool>>,

    #[account(
        init,
        payer = funder,
        token::token_program = token_program_a,
        token::mint = token_mint_a,
        token::authority = ai_dex_pool
    )]
    pub token_vault_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = funder,
        token::token_program = token_program_b,
        token::mint = token_mint_b,
        token::authority = ai_dex_pool
    )]
    pub token_vault_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(has_one = ai_dex_config, constraint = fee_tier.tick_spacing == tick_spacing)]
    pub fee_tier: Account<'info, FeeTier>,

    #[account(address = token_mint_a.to_account_info().owner.clone())]
    pub token_program_a: Interface<'info, TokenInterface>,
    #[account(address = token_mint_b.to_account_info().owner.clone())]
    pub token_program_b: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Initializes a new pool in the protocol.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts and programs required for the operation.
/// * `tick_spacing` - The spacing between ticks in the pool.
/// * `initial_sqrt_price` - The initial square root price of the pool.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the operation is successful, otherwise returns an error.
///
/// # Errors
///
/// * `ErrorCode::UnsupportedTokenMintError` - If the token mint is not supported.
pub fn initialize_pool_handler(
    ctx: Context<InitializePool>,
    tick_spacing: u16,
    initial_sqrt_price: u128,
) -> Result<()> {
    let token_mint_a = ctx.accounts.token_mint_a.key();
    let token_mint_b = ctx.accounts.token_mint_b.key();

    let ai_dex = &mut ctx.accounts.ai_dex_pool;
    let ai_dex_config = &ctx.accounts.ai_dex_config;

    let default_fee_rate = ctx.accounts.fee_tier.default_fee_rate;

    // ignore the bump passed and use one Anchor derived
    let bump = ctx.bumps.ai_dex_pool;

    // Don't allow creating a pool with unsupported token mints
    let is_token_wrapper_initialized_a = is_token_wrapper_initialized(
      ai_dex_config.key(),
      token_mint_a,
      &ctx.accounts.token_wrapper_a
    )?;

    if !is_supported_token_mint(&ctx.accounts.token_mint_a, is_token_wrapper_initialized_a).unwrap() {
      return Err(ErrorCode::UnsupportedTokenMintError.into());
    }

    let is_token_wrapper_initialized_b = is_token_wrapper_initialized(
      ai_dex_config.key(),
      token_mint_b,
      &ctx.accounts.token_wrapper_b
    )?;

    if !is_supported_token_mint(&ctx.accounts.token_mint_b, is_token_wrapper_initialized_b).unwrap() {
      return Err(ErrorCode::UnsupportedTokenMintError.into());
    }

    // Initialize the pool
    let result = ai_dex.initialize(
        ai_dex_config,
        bump,
        tick_spacing,
        initial_sqrt_price,
        default_fee_rate,
        token_mint_a,
        ctx.accounts.token_vault_a.key(),
        token_mint_b,
        ctx.accounts.token_vault_b.key(),
    );

    // Check for initialization errors
    match result {
        Ok(_) => {
            emit!(PoolInitializedEvent {
                ai_dex_pool: ai_dex.key(),
                ai_dex_config: ai_dex_config.key(),
                token_mint_a: token_mint_a,
                token_mint_b: token_mint_b,
                token_wrapper_a: ctx.accounts.token_wrapper_a.key(),
                token_wrapper_b: ctx.accounts.token_wrapper_b.key(),
                funder: ctx.accounts.funder.key(),
                tick_spacing,
                initial_sqrt_price,
                default_fee_rate,
                token_vault_a: ctx.accounts.token_vault_a.key(),
                token_vault_b: ctx.accounts.token_vault_b.key(),
                fee_tier: ctx.accounts.fee_tier.key(),
                token_program_a: ctx.accounts.token_program_a.key(),
                token_program_b: ctx.accounts.token_program_b.key(),
            });            
            Ok(())
        },
        Err(e) => {
            // Handle errors by returning the error and optionally logging it
            msg!("Error: Failed to initialize pool - {:?}", e);
            Err(e)
        }
    }
}
