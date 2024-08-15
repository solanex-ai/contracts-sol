use crate::{
    errors::ErrorCode,
    math::{
        tick_index_from_sqrt_price, MAX_FEE_RATE, MAX_PROTOCOL_FEE_RATE, MAX_SQRT_PRICE_X64,
        MIN_SQRT_PRICE_X64,
    },
};
use anchor_lang::prelude::*;

use super::AiDexConfig;

#[account]
#[derive(Default)]
/// Represents the state of the AiDex program.
pub struct AiDexPool {
    /// The configuration of the AiDex program.
    pub ai_dex_config: Pubkey, // 32

    /// The bump value used for account creation.
    pub ai_dex_bump: [u8; 1],   // 1

    /// The spacing between ticks.
    pub tick_spacing: u16,          // 2

    /// The seed value used for tick spacing.
    pub tick_spacing_seed: [u8; 2], // 2

    /// The fee rate for transactions, stored as hundredths of a basis point.
    /// A value of u16::MAX corresponds to approximately 6.5%.
    pub fee_rate: u16, // 2

    /// The portion of the fee rate taken as protocol fees, stored as basis points.
    pub protocol_fee_rate: u16, // 2

    /// The maximum amount that can be held by the Solana account.
    pub liquidity: u128, // 16

    /// The square root of the price, stored as Q64.64.
    pub sqrt_price: u128,        // 16

    /// The current index of the tick.
    pub tick_current_index: i32, // 4

    /// The protocol fee owed for token A.
    pub protocol_fee_owed_a: u64, // 8
    /// The protocol fee owed for token B.
    pub protocol_fee_owed_b: u64, // 8

    /// The mint address for token A.
    pub token_mint_a: Pubkey,  // 32
    /// The mint address for token B.
    pub token_mint_b: Pubkey,  // 32

    /// The vault address for token A.
    pub token_vault_a: Pubkey, // 32
    /// The vault address for token B.
    pub token_vault_b: Pubkey, // 32

    /// The global fee growth for token A, stored as Q64.64.
    pub fee_growth_global_a: u128, // 16
    /// The global fee growth for token B, stored as Q64.64.
    pub fee_growth_global_b: u128, // 16

    /// The timestamp when the rewards were last updated.
    pub reward_last_updated_timestamp: u64, // 8

    /// The reward information for each reward.
    pub reward_infos: [AiDexRewardInfo; NUM_REWARDS], // 384
}

// Number of rewards supported by AiDex
pub const NUM_REWARDS: usize = 3;

/// The AiDex struct represents the state of the AiDex program.
impl AiDexPool {
    /// The total length of the AiDex struct.
    pub const LEN: usize = 8 + 261 + 384;

    /// Returns an array of references to the seeds used for program address generation.
    pub fn seeds(&self) -> [&[u8]; 6] {
        [
            &b"ai_dex"[..],
            self.ai_dex_config.as_ref(),
            self.token_mint_a.as_ref(),
            self.token_mint_b.as_ref(),
            self.tick_spacing_seed.as_ref(),
            self.ai_dex_bump.as_ref(),
        ]
    }

    /// Returns the input token mint based on the given direction.
    ///
    /// # Parameters
    /// - `a_to_b` - A boolean indicating the direction of the swap.
    ///
    /// # Returns
    /// The input token mint.
    pub fn input_token_mint(&self, a_to_b: bool) -> Pubkey {
        if a_to_b {
            self.token_mint_a
        } else {
            self.token_mint_b
        }
    }

    /// Returns the input token vault based on the given direction.
    ///
    /// # Parameters
    /// - `a_to_b` - A boolean indicating the direction of the swap.
    ///
    /// # Returns
    /// The input token vault.
    pub fn input_token_vault(&self, a_to_b: bool) -> Pubkey {
        if a_to_b {
            self.token_vault_a
        } else {
            self.token_vault_b
        }
    }

    /// Returns the output token mint based on the given direction.
    ///
    /// # Parameters
    /// - `a_to_b` - A boolean indicating the direction of the swap.
    ///
    /// # Returns
    /// The output token mint.
    pub fn output_token_mint(&self, a_to_b: bool) -> Pubkey {
        if a_to_b {
            self.token_mint_b
        } else {
            self.token_mint_a
        }
    }

    /// Returns the output token vault based on the given direction.
    ///
    /// # Parameters
    /// - `a_to_b` - A boolean indicating the direction of the swap.
    ///
    /// # Returns
    /// The output token vault.
    pub fn output_token_vault(&self, a_to_b: bool) -> Pubkey {
        if a_to_b {
            self.token_vault_b
        } else {
            self.token_vault_a
        }
    }

    /// Initializes the AiDex struct with the provided parameters.
    ///
    /// # Parameters
    /// - `ai_dex_config` - The account containing the AiDex configuration.
    /// - `bump` - The bump value for program address generation.
    /// - `tick_spacing` - The tick spacing value.
    /// - `sqrt_price` - The square root price value.
    /// - `default_fee_rate` - The default fee rate value.
    /// - `token_mint_a` - The mint of token A.
    /// - `token_vault_a` - The vault of token A.
    /// - `token_mint_b` - The mint of token B.
    /// - `token_vault_b` - The vault of token B.
    ///
    /// # Errors
    /// This function returns an error if the token mint order is invalid or if the square root price is out of bounds.
    pub fn initialize(
        &mut self,
        ai_dex_config: &Account<AiDexConfig>,
        bump: u8,
        tick_spacing: u16,
        sqrt_price: u128,
        default_fee_rate: u16,
        token_mint_a: Pubkey,
        token_vault_a: Pubkey,
        token_mint_b: Pubkey,
        token_vault_b: Pubkey,
    ) -> Result<()> {
        // Check if the token mint order is valid
        if token_mint_a.ge(&token_mint_b) {
            return Err(ErrorCode::InvalidTokenMintOrderError.into());
        }

        // Check if the square root price is within bounds
        if sqrt_price < MIN_SQRT_PRICE_X64 || sqrt_price > MAX_SQRT_PRICE_X64 {
            return Err(ErrorCode::SqrtPriceOutOfBoundsError.into());
        }

        // Initialize the AiDex struct with the provided parameters
        self.ai_dex_config = ai_dex_config.key();
        self.ai_dex_bump = [bump];

        self.tick_spacing = tick_spacing;
        self.tick_spacing_seed = self.tick_spacing.to_le_bytes();

        self.update_fee_rate(default_fee_rate)?;
        self.update_protocol_fee_rate(ai_dex_config.default_protocol_fee_rate)?;

        self.liquidity = 0;
        self.sqrt_price = sqrt_price;
        self.tick_current_index = tick_index_from_sqrt_price(&sqrt_price);

        self.protocol_fee_owed_a = 0;
        self.protocol_fee_owed_b = 0;

        self.token_mint_a = token_mint_a;
        self.token_vault_a = token_vault_a;
        self.fee_growth_global_a = 0;

        self.token_mint_b = token_mint_b;
        self.token_vault_b = token_vault_b;
        self.fee_growth_global_b = 0;

        self.reward_infos =
            [AiDexRewardInfo::new(ai_dex_config.config_authority);
                NUM_REWARDS];

        Ok(())
    }

    /// Update all reward values for the AiDex.
    ///
    /// # Parameters
    /// - `reward_infos` - An array of all updated ai_dex rewards
    /// - `reward_last_updated_timestamp` - The timestamp when the rewards were last updated
    pub fn update_rewards(
        &mut self,
        reward_infos: [AiDexRewardInfo; NUM_REWARDS],
        reward_last_updated_timestamp: u64,
    ) {
        self.reward_last_updated_timestamp = reward_last_updated_timestamp;
        self.reward_infos = reward_infos;
    }

    /// Update the rewards and liquidity values for the AiDex.
    ///
    /// # Parameters
    /// - `reward_infos` - An array of all updated ai_dex rewards
    /// - `liquidity` - The updated liquidity value
    /// - `reward_last_updated_timestamp` - The timestamp when the rewards were last updated
    pub fn update_rewards_and_liquidity(
        &mut self,
        reward_infos: [AiDexRewardInfo; NUM_REWARDS],
        liquidity: u128,
        reward_last_updated_timestamp: u64,
    ) {
        self.update_rewards(reward_infos, reward_last_updated_timestamp);
        self.liquidity = liquidity;
    }

    /// Update the reward authority at the specified AiDex reward index.
    ///
    /// # Parameters
    /// - `index` - The index of the reward to update.
    /// - `authority` - The new authority for the reward.
    ///
    /// # Errors
    /// This function returns an error if the reward index is invalid.
    pub fn update_reward_authority(&mut self, index: usize, authority: Pubkey) -> Result<()> {
        if index >= NUM_REWARDS {
            return Err(ErrorCode::InvalidRewardIndexError.into());
        }
        self.reward_infos[index].authority = authority;

        Ok(())
    }

    /// Update the emissions for the specified AiDex reward index.
    ///
    /// # Parameters
    /// - `index` - The index of the reward to update.
    /// - `reward_infos` - An array of all updated ai_dex rewards.
    /// - `timestamp` - The timestamp when the emissions were last updated.
    /// - `emissions_per_second_x64` - The new emissions per second value.
    ///
    /// # Errors
    /// This function returns an error if the reward index is invalid.
    pub fn update_emissions(
        &mut self,
        index: usize,
        reward_infos: [AiDexRewardInfo; NUM_REWARDS],
        timestamp: u64,
        emissions_per_second_x64: u128,
    ) -> Result<()> {
        if index >= NUM_REWARDS {
            return Err(ErrorCode::InvalidRewardIndexError.into());
        }
        self.update_rewards(reward_infos, timestamp);
        self.reward_infos[index].emissions_per_second_x64 = emissions_per_second_x64;

        Ok(())
    }

    /// Initializes the reward at the specified AiDex reward index.
    ///
    /// # Parameters
    /// - `index` - The index of the reward to initialize.
    /// - `mint` - The mint of the reward.
    /// - `vault` - The vault of the reward.
    ///
    /// # Errors
    /// This function returns an error if the reward index is invalid or if there is already an initialized reward at a lower index.
    pub fn initialize_reward(&mut self, index: usize, mint: Pubkey, vault: Pubkey) -> Result<()> {
        if index >= NUM_REWARDS {
            return Err(ErrorCode::InvalidRewardIndexError.into());
        }

        let lowest_index = self.reward_infos.iter().position(|r| !r.initialized())
            .ok_or(ErrorCode::InvalidRewardIndexError)?;

        if lowest_index != index {
            return Err(ErrorCode::InvalidRewardIndexError.into());
        }

        self.reward_infos[index].mint = mint;
        self.reward_infos[index].vault = vault;

        Ok(())
    }

    /// Update the AiDex state after a swap.
    ///
    /// # Parameters
    /// - `liquidity` - The updated liquidity value.
    /// - `tick_index` - The updated tick index value.
    /// - `sqrt_price` - The updated square root price value.
    /// - `fee_growth_global` - The updated fee growth global value.
    /// - `reward_infos` - An array of all updated ai_dex rewards.
    /// - `protocol_fee` - The protocol fee value.
    /// - `is_token_fee_in_a` - A boolean indicating if the token fee is in token A.
    /// - `reward_last_updated_timestamp` - The timestamp when the rewards were last updated.
    pub fn update_after_swap(
        &mut self,
        liquidity: u128,
        tick_index: i32,
        sqrt_price: u128,
        fee_growth_global: u128,
        reward_infos: [AiDexRewardInfo; NUM_REWARDS],
        protocol_fee: u64,
        is_token_fee_in_a: bool,
        reward_last_updated_timestamp: u64,
    ) {
        self.tick_current_index = tick_index;
        self.sqrt_price = sqrt_price;
        self.liquidity = liquidity;
        self.reward_infos = reward_infos;
        self.reward_last_updated_timestamp = reward_last_updated_timestamp;
        if is_token_fee_in_a {
            // Add fees taken via a
            self.fee_growth_global_a = fee_growth_global;
            self.protocol_fee_owed_a += protocol_fee;
        } else {
            // Add fees taken via b
            self.fee_growth_global_b = fee_growth_global;
            self.protocol_fee_owed_b += protocol_fee;
        }
    }

    /// Update the fee rate for the AiDex.
    ///
    /// # Parameters
    /// - `fee_rate` - The new fee rate value.
    ///
    /// # Errors
    /// This function returns an error if the fee rate exceeds the maximum fee rate.
    pub fn update_fee_rate(&mut self, fee_rate: u16) -> Result<()> {
        if fee_rate > MAX_FEE_RATE {
            return Err(ErrorCode::FeeRateExceededError.into());
        }
        self.fee_rate = fee_rate;

        Ok(())
    }

    /// Update the protocol fee rate for the AiDex.
    ///
    /// # Parameters
    /// - `protocol_fee_rate` - The new protocol fee rate value.
    ///
    /// # Errors
    /// This function returns an error if the protocol fee rate exceeds the maximum protocol fee rate.
    pub fn update_protocol_fee_rate(&mut self, protocol_fee_rate: u16) -> Result<()> {
        if protocol_fee_rate > MAX_PROTOCOL_FEE_RATE {
            return Err(ErrorCode::ProtocolFeeRateExceededError.into());
        }
        self.protocol_fee_rate = protocol_fee_rate;

        Ok(())
    }

    /// Reset the protocol fees owed by the AiDex.
    pub fn reset_protocol_fees_owed(&mut self) {
        self.protocol_fee_owed_a = 0;
        self.protocol_fee_owed_b = 0;
    }
}

/// Stores the state relevant for tracking liquidity mining rewards at the `AiDex` level.
/// These values are used in conjunction with `PositionRewardInfo`, `Tick.reward_growths_outside`,
/// and `AiDex.reward_last_updated_timestamp` to determine how many rewards are earned by open
/// positions.
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Default, Debug, PartialEq)]
pub struct AiDexRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions.
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

impl AiDexRewardInfo {
    /// Creates a new `AiDexRewardInfo` with the authority set
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            ..Default::default()
        }
    }

    /// Returns true if this reward is initialized.
    /// Once initialized, a reward cannot transition back to uninitialized.
    pub fn initialized(&self) -> bool {
        self.mint.ne(&Pubkey::default())
    }

    /// Maps all reward data to only the reward growth accumulators
    pub fn to_reward_growths(
        reward_infos: &[AiDexRewardInfo; NUM_REWARDS],
    ) -> [u128; NUM_REWARDS] {
        let mut reward_growths = [0u128; NUM_REWARDS];
        for i in 0..NUM_REWARDS {
            reward_growths[i] = reward_infos[i].growth_global_x64;
        }
        reward_growths
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy)]
pub struct AiDexBumps {
    pub ai_dex_bump: u8,
}

#[test]
fn test_ai_dex_reward_info_not_initialized() {
    let reward_info = AiDexRewardInfo::default();
    assert_eq!(reward_info.initialized(), false);
}

#[test]
fn test_ai_dex_reward_info_initialized() {
    let reward_info = &mut AiDexRewardInfo::default();
    reward_info.mint = Pubkey::new_unique();
    assert_eq!(reward_info.initialized(), true);
}

#[cfg(test)]
pub mod ai_dex_builder {
    use super::{AiDexPool, AiDexRewardInfo, NUM_REWARDS};

    #[derive(Default)]
    pub struct AiDexBuilder {
        liquidity: u128,
        tick_spacing: u16,
        tick_current_index: i32,
        sqrt_price: u128,
        fee_rate: u16,
        protocol_fee_rate: u16,
        fee_growth_global_a: u128,
        fee_growth_global_b: u128,
        reward_last_updated_timestamp: u64,
        reward_infos: [AiDexRewardInfo; NUM_REWARDS],
    }

    impl AiDexBuilder {
        pub fn new() -> Self {
            Self {
                reward_infos: [AiDexRewardInfo::default(); NUM_REWARDS],
                ..Default::default()
            }
        }

        pub fn liquidity(mut self, liquidity: u128) -> Self {
            self.liquidity = liquidity;
            self
        }

        pub fn reward_last_updated_timestamp(mut self, reward_last_updated_timestamp: u64) -> Self {
            self.reward_last_updated_timestamp = reward_last_updated_timestamp;
            self
        }

        pub fn reward_info(mut self, index: usize, reward_info: AiDexRewardInfo) -> Self {
            self.reward_infos[index] = reward_info;
            self
        }

        pub fn reward_infos(mut self, reward_infos: [AiDexRewardInfo; NUM_REWARDS]) -> Self {
            self.reward_infos = reward_infos;
            self
        }

        pub fn tick_spacing(mut self, tick_spacing: u16) -> Self {
            self.tick_spacing = tick_spacing;
            self
        }

        pub fn tick_current_index(mut self, tick_current_index: i32) -> Self {
            self.tick_current_index = tick_current_index;
            self
        }

        pub fn sqrt_price(mut self, sqrt_price: u128) -> Self {
            self.sqrt_price = sqrt_price;
            self
        }

        pub fn fee_growth_global_a(mut self, fee_growth_global_a: u128) -> Self {
            self.fee_growth_global_a = fee_growth_global_a;
            self
        }

        pub fn fee_growth_global_b(mut self, fee_growth_global_b: u128) -> Self {
            self.fee_growth_global_b = fee_growth_global_b;
            self
        }

        pub fn fee_rate(mut self, fee_rate: u16) -> Self {
            self.fee_rate = fee_rate;
            self
        }

        pub fn protocol_fee_rate(mut self, protocol_fee_rate: u16) -> Self {
            self.protocol_fee_rate = protocol_fee_rate;
            self
        }

        pub fn build(self) -> AiDexPool {
            AiDexPool {
                liquidity: self.liquidity,
                reward_last_updated_timestamp: self.reward_last_updated_timestamp,
                reward_infos: self.reward_infos,
                tick_current_index: self.tick_current_index,
                sqrt_price: self.sqrt_price,
                tick_spacing: self.tick_spacing,
                fee_growth_global_a: self.fee_growth_global_a,
                fee_growth_global_b: self.fee_growth_global_b,
                fee_rate: self.fee_rate,
                protocol_fee_rate: self.protocol_fee_rate,
                ..Default::default()
            }
        }
    }
}
