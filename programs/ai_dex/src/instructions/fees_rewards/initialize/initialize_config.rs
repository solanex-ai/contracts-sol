use anchor_lang::prelude::*;

use crate::state::*;

#[event]
pub struct ConfigInitializedEvent {
    pub config_key: Pubkey,
    pub funder: Pubkey,
    pub config_authority: Pubkey,
    pub default_protocol_fee_rate: u16,
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(init, payer = funder, space = AiDexConfig::LEN)]
    pub config: Account<'info, AiDexConfig>,

    #[account(mut)]
    pub funder: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Initializes the configuration for the protocol.
///
/// This function handles the initialization of the protocol configuration. It sets up the
/// authorities for fee collection, protocol fee collection, and reward emissions, as well as
/// the default protocol fee rate.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts required for initializing the configuration.
/// * `fee_authority` - The public key of the fee authority.
/// * `collect_protocol_fees_authority` - The public key of the authority responsible for collecting protocol fees.
/// * `reward_emissions_super_authority` - The public key of the super authority for reward emissions.
/// * `default_protocol_fee_rate` - The default protocol fee rate to be set.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the configuration is successfully initialized,
/// or an `Err` if an error occurs.
pub fn initialize_config_handler(
    ctx: Context<InitializeConfig>,
    config_authority: Pubkey,
    default_protocol_fee_rate: u16,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.initialize(
        config_authority,
        default_protocol_fee_rate,
    )?;

    emit!(ConfigInitializedEvent {
        config_key: config.key(),
        funder: ctx.accounts.funder.key(),
        config_authority,
        default_protocol_fee_rate,
    });
    
    Ok(())
}
