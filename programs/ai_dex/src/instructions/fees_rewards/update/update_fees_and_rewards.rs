use anchor_lang::prelude::*;

use crate::{
    orchestrator::liquidity_orchestrator::calculate_fee_and_reward_growths, state::*, util::to_timestamp_u64,
};

#[event]
pub struct FeesAndRewardsUpdatedEvent {
    pub ai_dex: Pubkey,
    pub position: PositionData,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub reward_infos: Vec<ai_dex::AiDexRewardInfo>,
    pub timestamp: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PositionData {
    pub key: Pubkey,
    pub update_info: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardInfo {
    pub reward_type: String,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct UpdateFeesAndRewards<'info> {
    #[account(mut)]
    pub ai_dex_pool: Account<'info, AiDexPool>,

    #[account(mut, has_one = ai_dex_pool)]
    pub position: Account<'info, Position>,

    #[account(has_one = ai_dex_pool)]
    pub tick_array_lower: AccountLoader<'info, TickArray>,
    #[account(has_one = ai_dex_pool)]
    pub tick_array_upper: AccountLoader<'info, TickArray>,
}

/// Updates the fees and rewards for a given position.
///
/// This function handles the update of fees and rewards for a specific position in the AI DEX.
/// It calculates the fee and reward growths based on the current state and updates the position
/// and AI DEX accordingly.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts required for updating the fees and rewards.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the fees and rewards are successfully updated,
/// or an `Err` if an error occurs.
pub fn update_fees_and_rewards_handler(ctx: Context<UpdateFeesAndRewards>) -> Result<()> {
    let ai_dex = &mut ctx.accounts.ai_dex_pool;
    let position = &mut ctx.accounts.position;
    let timestamp = to_timestamp_u64(Clock::get()?.unix_timestamp)?;

    let (position_update, reward_infos) = calculate_fee_and_reward_growths(
        ai_dex,
        position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        timestamp,
    )?;

    ai_dex.update_rewards(reward_infos, timestamp);
    position.update(&position_update);

    emit!(FeesAndRewardsUpdatedEvent {
        ai_dex: ctx.accounts.ai_dex_pool.key(),
        position: PositionData {
            key: ctx.accounts.position.key(),
            update_info: format!("{:?}", position_update), // Convert `position_update` to a string or other appropriate format
        },
        tick_array_lower: ctx.accounts.tick_array_lower.key(),
        tick_array_upper: ctx.accounts.tick_array_upper.key(),
        reward_infos: reward_infos.to_vec(),
        timestamp,
    });
    
    Ok(())
}
