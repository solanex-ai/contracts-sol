use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

use crate::errors::ErrorCode;
use crate::orchestrator::ai_dex_orchestrator::next_ai_dex_reward_infos;
use crate::math::checked_mul_shift_right;
use crate::state::AiDexPool;
use crate::util::to_timestamp_u64;

const DAY_IN_SECONDS: u128 = 60 * 60 * 24;

#[event]
pub struct RewardEmissionsSetEvent {
    pub ai_dex_pool: Pubkey,
    pub reward_index: u8,
    pub reward_authority: Pubkey,
    pub reward_vault: RewardVaultData,
    pub emissions_per_second_x64: u128,
    pub emissions_per_day: u64,
    pub timestamp: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardVaultData {
    pub key: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct SetRewardEmissions<'info> {
    #[account(mut)]
    pub ai_dex_pool: Account<'info, AiDexPool>,

    #[account(address = ai_dex_pool.reward_infos[reward_index as usize].authority)]
    pub reward_authority: Signer<'info>,

    #[account(address = ai_dex_pool.reward_infos[reward_index as usize].vault)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
}

/// Sets the reward emissions for the protocol.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts and programs required for the operation.
/// * `reward_index` - The index of the reward to set emissions for.
/// * `emissions_per_second_x64` - The emissions rate per second, scaled by 2^64.
///
/// # Returns
///
/// * `Result<()>` - Returns an Ok result if the operation is successful, otherwise returns an error.
///
/// # Errors
///
/// * `ErrorCode::InsufficientRewardVaultAmountError` - If the reward vault does not have enough tokens to cover the emissions for a day.
pub fn set_reward_emissions_handler(
    ctx: Context<SetRewardEmissions>,
    reward_index: u8,
    emissions_per_second_x64: u128,
) -> Result<()> {
    let ai_dex = &ctx.accounts.ai_dex_pool;
    let reward_vault = &ctx.accounts.reward_vault;

    let emissions_per_day = checked_mul_shift_right(DAY_IN_SECONDS, emissions_per_second_x64)?;
    if reward_vault.amount < emissions_per_day {
        return Err(ErrorCode::InsufficientRewardVaultAmountError.into());
    }

    let timestamp = to_timestamp_u64(Clock::get()?.unix_timestamp)?;
    let next_reward_infos = next_ai_dex_reward_infos(ai_dex, timestamp)?;

    ctx.accounts.ai_dex_pool.update_emissions(
        reward_index as usize,
        next_reward_infos,
        timestamp,
        emissions_per_second_x64,
    )?;

    emit!(RewardEmissionsSetEvent {
        ai_dex_pool: ctx.accounts.ai_dex_pool.key(),
        reward_index,
        reward_authority: ctx.accounts.reward_authority.key(),
        reward_vault: RewardVaultData {
            key: reward_vault.key(),
            amount: reward_vault.amount,
        },
        emissions_per_second_x64,
        emissions_per_day,
        timestamp,
    });
    

    Ok(())
}
