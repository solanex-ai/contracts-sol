use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::memo::Memo;

use crate::errors::ErrorCode;
use crate::orchestrator::liquidity_orchestrator::{
    calculate_liquidity_token_deltas, calculate_modify_liquidity, sync_modify_liquidity_values,
};
use crate::math::convert_to_liquidity_delta;
use crate::state::*;
use crate::util::{calculate_transfer_fee_included_amount, parse_remaining_accounts, AccountsType, RemainingAccountsInfo};
use crate::util::{to_timestamp_u64, transfer_from_owner_to_vault, verify_position_authority};

#[event]
pub struct IncreaseLiquidityEvent {
    pub liquidity_amount: u128,
    pub token_max_a: u64,
    pub token_max_b: u64,
    pub position_authority: Pubkey,
    pub ai_dex_pool: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub token_owner_account_a: Pubkey,
    pub token_owner_account_b: Pubkey,
    pub delta_a: u64,
    pub delta_b: u64,
    pub transfer_fee_included_delta_a: u64,
    pub transfer_fee_included_delta_b: u64,
    pub timestamp: u64,
}

#[derive(Accounts)]
pub struct ModifyLiquidity<'info> {
    #[account(mut)]
    pub ai_dex_pool: Account<'info, AiDexPool>,

    #[account(address = token_mint_a.to_account_info().owner.clone())]
    pub token_program_a: Interface<'info, TokenInterface>,
    #[account(address = token_mint_b.to_account_info().owner.clone())]
    pub token_program_b: Interface<'info, TokenInterface>,

    pub memo_program: Program<'info, Memo>,

    pub position_authority: Signer<'info>,

    #[account(mut, has_one = ai_dex_pool)]
    pub position: Account<'info, Position>,
    #[account(
        constraint = position_token_account.mint == position.position_mint,
        constraint = position_token_account.amount == 1
    )]
    pub position_token_account: Box<Account<'info, token::TokenAccount>>,

    #[account(address = ai_dex_pool.token_mint_a)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,
    #[account(address = ai_dex_pool.token_mint_b)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(mut, constraint = token_owner_account_a.mint == ai_dex_pool.token_mint_a)]
    pub token_owner_account_a: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, constraint = token_owner_account_b.mint == ai_dex_pool.token_mint_b)]
    pub token_owner_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, constraint = token_vault_a.key() == ai_dex_pool.token_vault_a)]
    pub token_vault_a: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, constraint = token_vault_b.key() == ai_dex_pool.token_vault_b)]
    pub token_vault_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, has_one = ai_dex_pool)]
    pub tick_array_lower: AccountLoader<'info, TickArray>,
    #[account(mut, has_one = ai_dex_pool)]
    pub tick_array_upper: AccountLoader<'info, TickArray>,

}

/// Handles the increase of liquidity in the protocol.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts and programs required for the operation.
/// * `liquidity_amount` - The amount of liquidity to be added.
/// * `token_max_a` - The maximum amount of token A that can be transferred.
/// * `token_max_b` - The maximum amount of token B that can be transferred.
/// * `remaining_accounts_info` - Optional information about remaining accounts.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the operation is successful, otherwise returns an error.
///
/// # Errors
///
/// * `ErrorCode::ZeroLiquidityError` - If the liquidity amount is zero.
/// * `ErrorCode::TokenLimitExceededError` - If the transfer amount exceeds the specified token limits.
pub fn increase_liquidity_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ModifyLiquidity<'info>>,
    liquidity_amount: u128,
    token_max_a: u64,
    token_max_b: u64,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Result<()> {
    verify_position_authority(
        &ctx.accounts.position_token_account,
        &ctx.accounts.position_authority,
    )?;

    if liquidity_amount == 0 {
        return Err(ErrorCode::ZeroLiquidityError.into());
    }

    let timestamp = to_timestamp_u64(Clock::get()?.unix_timestamp)?;

    let remaining_accounts = parse_remaining_accounts(
        &ctx.remaining_accounts,
        &remaining_accounts_info,
        &[AccountsType::TransferHookA, AccountsType::TransferHookB],
    )?;

    let liquidity_delta = convert_to_liquidity_delta(liquidity_amount, true)?;

    let update = calculate_modify_liquidity(
        &ctx.accounts.ai_dex_pool,
        &ctx.accounts.position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        liquidity_delta,
        timestamp,
    )?;

    sync_modify_liquidity_values(
        &mut ctx.accounts.ai_dex_pool,
        &mut ctx.accounts.position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        update,
        timestamp,
    )?;

    let (delta_a, delta_b) = calculate_liquidity_token_deltas(
        ctx.accounts.ai_dex_pool.tick_current_index,
        ctx.accounts.ai_dex_pool.sqrt_price,
        &ctx.accounts.position,
        liquidity_delta,
    )?;

    let transfer_fee_included_delta_a = calculate_transfer_fee_included_amount(
        &ctx.accounts.token_mint_a,
        delta_a,
    )?;
    let transfer_fee_included_delta_b = calculate_transfer_fee_included_amount(
        &ctx.accounts.token_mint_b,
        delta_b,
    )?;

    // token_max_a and token_max_b should be applied to the transfer fee included amount
    if transfer_fee_included_delta_a.amount > token_max_a {
        return Err(ErrorCode::TokenLimitExceededError.into());
    }
    if transfer_fee_included_delta_b.amount > token_max_b {
        return Err(ErrorCode::TokenLimitExceededError.into());
    }

    transfer_from_owner_to_vault(
        &ctx.accounts.position_authority,
        &ctx.accounts.token_mint_a,
        &ctx.accounts.token_owner_account_a,
        &ctx.accounts.token_vault_a,
        &ctx.accounts.token_program_a,
        &ctx.accounts.memo_program,
        &remaining_accounts.transfer_hook_a,
        transfer_fee_included_delta_a.amount,
    )?;

    transfer_from_owner_to_vault(
        &ctx.accounts.position_authority,
        &ctx.accounts.token_mint_b,
        &ctx.accounts.token_owner_account_b,
        &ctx.accounts.token_vault_b,
        &ctx.accounts.token_program_b,
        &ctx.accounts.memo_program,
        &remaining_accounts.transfer_hook_b,
        transfer_fee_included_delta_b.amount,
    )?;

    emit!(IncreaseLiquidityEvent {
        liquidity_amount,
        token_max_a,
        token_max_b,
        position_authority: ctx.accounts.position_authority.key(),
        ai_dex_pool: ctx.accounts.ai_dex_pool.key(),
        token_mint_a: ctx.accounts.token_mint_a.key(),
        token_mint_b: ctx.accounts.token_mint_b.key(),
        token_vault_a: ctx.accounts.token_vault_a.key(),
        token_vault_b: ctx.accounts.token_vault_b.key(),
        token_owner_account_a: ctx.accounts.token_owner_account_a.key(),
        token_owner_account_b: ctx.accounts.token_owner_account_b.key(),
        delta_a,
        delta_b,
        transfer_fee_included_delta_a: transfer_fee_included_delta_a.amount,
        transfer_fee_included_delta_b: transfer_fee_included_delta_b.amount,
        timestamp,
    });    

    Ok(())
}
