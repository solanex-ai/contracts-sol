use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::orchestrator::liquidity_orchestrator::{
    calculate_liquidity_token_deltas, calculate_modify_liquidity, sync_modify_liquidity_values,
};
use crate::math::convert_to_liquidity_delta;
use crate::util::{calculate_transfer_fee_excluded_amount, parse_remaining_accounts, AccountsType, RemainingAccountsInfo};
use crate::util::{to_timestamp_u64, transfer_from_vault_to_owner, verify_position_authority};
use crate::constants::transfer_memo;

use super::ModifyLiquidity;

#[event]
pub struct DecreaseLiquidityEvent {
    pub liquidity_amount: u128,
    pub token_min_a: u64,
    pub token_min_b: u64,
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
    pub transfer_fee_excluded_delta_a: u64,
    pub transfer_fee_excluded_delta_b: u64,
    pub timestamp: u64,
}

/// Handles the decrease of liquidity in the protocol.
///
/// This function verifies the position authority, processes the remaining accounts,
/// calculates the liquidity delta, and transfers the appropriate amounts from the vault
/// to the owner's accounts.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts required for the liquidity modification.
/// * `liquidity_amount` - The amount of liquidity to be decreased.
/// * `token_min_a` - The minimum amount of token A to be transferred.
/// * `token_min_b` - The minimum amount of token B to be transferred.
/// * `remaining_accounts_info` - Optional information about remaining accounts.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the liquidity decrease is successful, otherwise returns an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The position authority verification fails.
/// * The liquidity amount is zero.
/// * Parsing the remaining accounts fails.
/// * Calculating the liquidity delta fails.
/// * Calculating the modify liquidity values fails.
/// * Synchronizing the modify liquidity values fails.
/// * Calculating the liquidity token deltas fails.
/// * Calculating the transfer fee excluded amounts fails.
/// * The transfer fee excluded amounts are below the minimum thresholds.
/// * Transferring from the vault to the owner's accounts fails.
pub fn decrease_liquidity_handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ModifyLiquidity<'info>>,
    liquidity_amount: u128,
    token_min_a: u64,
    token_min_b: u64,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Result<()> {
    // Verify position authority
    verify_position_authority(
        &ctx.accounts.position_token_account,
        &ctx.accounts.position_authority,
    )?;

    // Check for zero liquidity amount
    if liquidity_amount == 0 {
        return Err(ErrorCode::ZeroLiquidityError.into());
    }

    // Get the current clock timestamp
    let timestamp = to_timestamp_u64(Clock::get()?.unix_timestamp)?;

    // Process remaining accounts
    let remaining_accounts = parse_remaining_accounts(
        &ctx.remaining_accounts,
        &remaining_accounts_info,
        &[
            AccountsType::TransferHookA,
            AccountsType::TransferHookB,
        ],
    )?;

    // Calculate liquidity delta
    let liquidity_delta = convert_to_liquidity_delta(liquidity_amount, false)?;

    // Calculate and sync modify liquidity values
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

    // Calculate liquidity token deltas
    let (delta_a, delta_b) = calculate_liquidity_token_deltas(
        ctx.accounts.ai_dex_pool.tick_current_index,
        ctx.accounts.ai_dex_pool.sqrt_price,
        &ctx.accounts.position,
        liquidity_delta,
    )?;

    // Calculate transfer fee excluded amounts
    let transfer_fee_excluded_delta_a = calculate_transfer_fee_excluded_amount(
        &ctx.accounts.token_mint_a,
        delta_a
    )?;
    let transfer_fee_excluded_delta_b = calculate_transfer_fee_excluded_amount(
        &ctx.accounts.token_mint_b,
        delta_b
    )?;

    // Check if transfer fee excluded amounts are above minimum thresholds
    if transfer_fee_excluded_delta_a.amount < token_min_a {
        return Err(ErrorCode::TokenAmountBelowMinimumError.into());
    }
    if transfer_fee_excluded_delta_b.amount < token_min_b {
        return Err(ErrorCode::TokenAmountBelowMinimumError.into());
    }

    // Transfer from vault to owner for token A
    transfer_from_vault_to_owner(
        &ctx.accounts.ai_dex_pool,
        &ctx.accounts.token_mint_a,
        &ctx.accounts.token_vault_a,
        &ctx.accounts.token_owner_account_a,
        &ctx.accounts.token_program_a,
        &ctx.accounts.memo_program,
        &remaining_accounts.transfer_hook_a,
        delta_a,
        transfer_memo::TRANSFER_MEMO_DECREASE_LIQUIDITY.as_bytes(),
    )?;

    // Transfer from vault to owner for token B
    transfer_from_vault_to_owner(
        &ctx.accounts.ai_dex_pool,
        &ctx.accounts.token_mint_b,
        &ctx.accounts.token_vault_b,
        &ctx.accounts.token_owner_account_b,
        &ctx.accounts.token_program_b,
        &ctx.accounts.memo_program,
        &remaining_accounts.transfer_hook_b,
        delta_b,
        transfer_memo::TRANSFER_MEMO_DECREASE_LIQUIDITY.as_bytes(),
    )?;

    emit!(DecreaseLiquidityEvent {
        liquidity_amount,
        token_min_a,
        token_min_b,
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
        transfer_fee_excluded_delta_a: transfer_fee_excluded_delta_a.amount,
        transfer_fee_excluded_delta_b: transfer_fee_excluded_delta_b.amount,
        timestamp,
    });

    Ok(())
}