use anchor_lang::prelude::*;

use crate::state::AiDexPool;

#[event]
pub struct RewardAuthorityUpdatedEvent {
    pub ai_dex_key: Pubkey,
    pub reward_index: u8,
    pub previous_reward_authority: Pubkey,
    pub new_reward_authority: Pubkey,
}

#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct SetRewardAuthority<'info> {
    #[account(mut)]
    pub ai_dex_pool: Account<'info, AiDexPool>,

    #[account(address = ai_dex_pool.reward_infos[reward_index as usize].authority)]
    pub reward_authority: Signer<'info>,

    /// CHECK: the account that will be new authority can be arbitrary
    pub new_reward_authority: UncheckedAccount<'info>,
}

/// Sets the reward authority for a specific reward index in the AI DEX.
///
/// This function updates the reward authority for the given reward index in the AI DEX configuration.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts required for setting the new reward authority.
/// * `reward_index` - The index of the reward for which the authority is to be updated.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the reward authority is successfully updated,
/// or an `Err` if an error occurs.
pub fn set_reward_authority_handler(
    ctx: Context<SetRewardAuthority>,
    reward_index: u8
) -> Result<()> {
    ctx.accounts.ai_dex_pool.update_reward_authority(
        reward_index as usize,
        ctx.accounts.new_reward_authority.key(),
    )?;

    emit!(RewardAuthorityUpdatedEvent {
        ai_dex_key: ctx.accounts.ai_dex_pool.key(),
        reward_index,
        previous_reward_authority: ctx.accounts.reward_authority.key(),
        new_reward_authority: ctx.accounts.new_reward_authority.key(),
    });    

    Ok(())
}
