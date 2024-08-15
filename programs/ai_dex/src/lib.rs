//! A CLMM contract for Dex AI.
use anchor_lang::prelude::*;

declare_id!("aij9zGKP31THhYTKFVbkrbzXdWywBSvFJYi6TSWqkzE");
#[doc(hidden)]
pub mod constants;
#[doc(hidden)]
pub mod errors;
#[doc(hidden)]
pub mod instructions;
#[doc(hidden)]
pub mod orchestrator;
#[doc(hidden)]
pub mod math;
pub mod state;
#[doc(hidden)]
pub mod tests;
#[doc(hidden)]
pub mod util;
#[doc(hidden)]
pub mod security;

use crate::state::{OpenPositionBumps, OpenPositionWithMetadataBumps};
use crate::util::RemainingAccountsInfo;
use instructions::*;

#[program]
pub mod ai_dex {
    use super::*;

    /// Initializes the configuration for the ai dex.
    ///
    /// This function sets up the initial configuration parameters for the protocol,
    /// including authorities and fee rates.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializeConfig` instruction.
    /// * `config_authority` - The public key of the authority responsible for managing.
    /// * `default_protocol_fee_rate` - The default fee rate for the protocol, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the initialization is successful,
    /// or an error if it fails.
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        config_authority: Pubkey,
        default_protocol_fee_rate: u16,
    ) -> Result<()> {
        return instructions::initialize_config::initialize_config_handler(
            ctx,
            config_authority,
            default_protocol_fee_rate,
        );
    }

    /// Initializes a new tick array with the given start tick index.
    ///
    /// This function sets up a new tick array starting at the specified tick index.
    /// It uses the provided context to initialize the tick array.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializeTickArray` instruction.
    /// * `start_tick_index` - The starting index for the tick array, represented as an `i32`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the tick array initialization is successful,
    /// or an error if it fails.
    pub fn initialize_tick_array(
        ctx: Context<InitializeTickArray>,
        start_tick_index: i32,
    ) -> Result<()> {
        return instructions::initialize_tick_array::initialize_tick_array_handler(ctx, start_tick_index);
    }

    /// Initializes a new fee tier with the given parameters.
    ///
    /// This function sets up a new fee tier with the specified tick spacing and default fee rate.
    /// It uses the provided context to initialize the fee tier.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializeFeeTier` instruction.
    /// * `tick_spacing` - The spacing between ticks in the fee tier, represented as a `u16`.
    /// * `default_fee_rate` - The default fee rate for the fee tier, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the fee tier initialization is successful,
    /// or an error if it fails.
    pub fn initialize_fee_tier(
        ctx: Context<InitializeFeeTier>,
        tick_spacing: u16,
        default_fee_rate: u16,
    ) -> Result<()> {
        return instructions::initialize_fee_tier::initialize_fee_tier_handler(
            ctx,
            tick_spacing,
            default_fee_rate
        );
    }

    /// Opens a new position within the specified tick range. NFT will be minted to represent the position.
    ///
    /// This function sets up a new position with the given lower and upper tick indices.
    /// It uses the provided context and bump seeds to initialize the position.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `OpenPosition` instruction.
    /// * `bumps` - The bump seeds for the position's PDA (Program Derived Address).
    /// * `tick_lower_index` - The lower tick index for the position, represented as an `i32`.
    /// * `tick_upper_index` - The upper tick index for the position, represented as an `i32`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the position is successfully opened,
    /// or an error if it fails.
    pub fn open_position(
        ctx: Context<OpenPosition>,
        bumps: OpenPositionBumps,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        return instructions::open_position::open_position_handler(
            ctx,
            bumps,
            tick_lower_index,
            tick_upper_index,
        );
    }

    /// Opens a new position with metadata within the specified tick range.
    /// NFT will be minted to represent the position.
    ///
    /// This function sets up a new position with the given lower and upper tick indices,
    /// and includes additional Metaplex metadata to identify the token. 
    /// It uses the provided context and bump seeds to initialize the position.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `OpenPositionWithMetadata` instruction.
    /// * `bumps` - The bump seeds for the position's PDA (Program Derived Address).
    /// * `tick_lower_index` - The lower tick index for the position, represented as an `i32`.
    /// * `tick_upper_index` - The upper tick index for the position, represented as an `i32`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the position is successfully opened,
    /// or an error if it fails.
    pub fn open_position_with_metadata(
        ctx: Context<OpenPositionWithMetadata>,
        bumps: OpenPositionWithMetadataBumps,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        return instructions::open_position_with_metadata::open_position_with_metadata_handler(
            ctx,
            bumps,
            tick_lower_index,
            tick_upper_index,
        );
    }

    /// Updates the fees and rewards for a position.
    ///
    /// This function updates the fees and rewards for the specified context.
    /// It uses the provided context to perform the update.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `UpdateFeesAndRewards` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the fees and rewards are successfully updated,
    /// or an error if it fails.
    pub fn update_fees_and_rewards(ctx: Context<UpdateFeesAndRewards>) -> Result<()> {
        return instructions::update_fees_and_rewards::update_fees_and_rewards_handler(ctx);
    }

    /// Closes an existing position in the ai dex pool.
    ///
    /// This function closes an existing position using the provided context.
    /// It ensures that the position is properly closed and any associated resources are released.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `ClosePosition` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the position is successfully closed,
    /// or an error if it fails.
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        return instructions::close_position::close_position_handler(ctx);
    }

    /// Sets the default fee rate for the fee tier.
    ///
    /// It uses the provided context (fee authority) and fee rate to update the default fee rate.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `SetDefaultFeeRate` instruction.
    /// * `default_fee_rate` - The default fee rate to set, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the default fee rate is successfully set,
    /// or an error if it fails.
    pub fn set_default_fee_rate(
        ctx: Context<SetDefaultFeeRate>,
        default_fee_rate: u16,
    ) -> Result<()> {
        return instructions::set_default_fee_rate::set_default_fee_rate_handler(ctx, default_fee_rate);
    }

    /// Sets the default protocol fee rate for the ai dex config.
    /// It uses the provided context (fee authority) and fee rate to update the default protocol fee rate.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `SetDefaultProtocolFeeRate` instruction.
    /// * `default_protocol_fee_rate` - The default protocol fee rate to set, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the default protocol fee rate is successfully set,
    /// or an error if it fails.
    pub fn set_default_protocol_fee_rate(
        ctx: Context<SetDefaultProtocolFeeRate>,
        default_protocol_fee_rate: u16,
    ) -> Result<()> {
        return instructions::set_default_protocol_fee_rate::set_default_protocol_fee_rate_handler(
            ctx,
            default_protocol_fee_rate,
        );
    }

    /// Sets the fee rate for the ai_dex.
    ///
    /// The fee rate is represented as hundredths of a basis point.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context (fee authority) for the `SetFeeRate` instruction.
    /// * `fee_rate` - The fee rate to set, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the fee rate is successfully set,
    /// or an error if it fails.
    pub fn set_fee_rate(ctx: Context<SetFeeRate>, fee_rate: u16) -> Result<()> {
        return instructions::set_fee_rate::set_fee_rate_handler(ctx, fee_rate);
    }

    /// Sets the protocol fee rate for an ai_dex.
    ///
    /// This function sets the protocol fee rate for the specified ai_dex.
    /// The protocol fee rate is represented as a basis point.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context (fee authority) for the `SetProtocolFeeRate` instruction.
    /// * `protocol_fee_rate` - The protocol fee rate to set, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the protocol fee rate is successfully set,
    /// or an error if it fails.
    pub fn set_protocol_fee_rate(
        ctx: Context<SetProtocolFeeRate>,
        protocol_fee_rate: u16,
    ) -> Result<()> {
        return instructions::set_protocol_fee_rate::set_protocol_fee_rate_handler(ctx, protocol_fee_rate);
    }

    /// Sets the fee authority for an ai dex config.
    /// The fee authority can set the fee and protocol fee rate for individual pools or
    /// set the default fee rate for newly minted pools.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `SetFeeAuthority` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the fee authority is successfully set,
    /// or an error if it fails.
    pub fn set_fee_authority(ctx: Context<SetFeeAuthority>) -> Result<()> {
        return instructions::set_fee_authority::set_fee_authority_handler(ctx);
    }

    /// Sets the ai dex pool reward authority for a specific reward index.
    ///
    /// This function sets the reward authority for the specified reward index in the context.
    /// The reward authority can manage the distribution and collection of rewards for the given index.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `SetRewardAuthority` instruction.
    /// * `reward_index` - The index of the reward to set the authority for, represented as a `u8`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the reward authority is successfully set,
    /// or an error if it fails.
    pub fn set_reward_authority(ctx: Context<SetRewardAuthority>, reward_index: u8) -> Result<()> {
        return instructions::set_reward_authority::set_reward_authority_handler(ctx, reward_index);
    }

    /// Sets the reward authority for a specific reward index by a super authority.
    ///
    /// The super authority has the power to manage the distribution
    /// and collection of rewards for the given index.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `SetRewardAuthorityBySuperAuthority` instruction.
    /// * `reward_index` - The index of the reward to set the authority for, represented as a `u8`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the reward authority is successfully set,
    /// or an error if it fails.
    pub fn set_reward_authority_by_config_authority(
        ctx: Context<SetRewardAuthorityByConfigAuthority>,
        reward_index: u8,
    ) -> Result<()> {
        return instructions::set_reward_authority_by_config_authority::set_reward_authority_by_config_authority_handler(ctx, reward_index);
    }

    /// Initializes a position trade batch.
    /// An NFT will be minted to represent the position trade batch in the user's wallet.
    ///
    /// It sets up the necessary state and configurations for a batch of trades.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializePositionTradeBatch` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the position trade batch is successfully initialized,
    /// or an error if it fails.
    pub fn initialize_position_trade_batch(ctx: Context<InitializePositionTradeBatch>) -> Result<()> {
        return instructions::initialize_trade_batch_position::initialize_trade_batch_position_handler(ctx);
    }

    /// Initializes a position trade batch with metaplex metadata.
    ///
    /// It sets up the necessary state and configurations for a batch of trades, 
    /// including additional METAPLEX metadata to identify the token.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializePositionTradeBatchWithMetadata` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the position trade batch with metadata is successfully initialized,
    /// or an error if it fails.
    pub fn initialize_position_trade_batch_with_metadata(
        ctx: Context<InitializePositionTradeBatchWithMetadata>,
    ) -> Result<()> {
        return instructions::initialize_trade_batch_position_with_metadata::initialize_trade_batch_position_with_metadata_handler(ctx);
    }

    /// Delete a PositionTradeBatch account. Burns the position trade batch token in the owner's wallet.
    ///
    /// ### Authority
    /// - `position_trade_batch_owner` - The owner that owns the position trade batch token.
    ///
    /// ### Special Errors
    /// - `NonDeletablePositionTradeBatchError` - The provided position trade batch has open positions.
    pub fn delete_position_trade_batch(ctx: Context<DeletePositionTradeBatch>) -> Result<()> {
        return instructions::delete_trade_batch_position::delete_trade_batch_position_handler(ctx);
    }

    /// Opens a trade batch position in an ai_dex pool.
    ///
    /// No new tokens are issued because the owner of the position trade batch becomes the owner of the position.
    /// The position will start off with 0 liquidity.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `OpenTradeBatchPosition` instruction.
    /// * `trade_batch_index` - The index of the trade batch to open the position for, represented as a `u16`.
    /// * `tick_lower_index` - The lower tick index for the position, represented as an `i32`.
    /// * `tick_upper_index` - The upper tick index for the position, represented as an `i32`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the trade batch position is successfully opened,
    /// or an error if it fails.
    pub fn open_trade_batch_position(
        ctx: Context<OpenTradeBatchPosition>,
        trade_batch_index: u16,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        return instructions::open_trade_batch_position::open_trade_batch_position_handler(
            ctx,
            trade_batch_index,
            tick_lower_index,
            tick_upper_index,
        );
    }

    /// Closes a trade batch position in the ai dex pool.
    ///
    /// This function closes a trade batch position using the provided context and trade batch index.
    /// It finalizes the position and performs any necessary cleanup.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `CloseTradeBatchPosition` instruction.
    /// * `trade_batch_index` - The index of the trade batch to close the position for, represented as a `u16`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the trade batch position is successfully closed,
    /// or an error if it fails.
    pub fn close_trade_batch_position(
        ctx: Context<CloseTradeBatchPosition>,
        trade_batch_index: u16,
    ) -> Result<()> {
        return instructions::close_trade_batch_position::close_trade_batch_position_handler(ctx, trade_batch_index);
    }

    /// Collects fees of the protocol.
    ///
    /// This function collects fees using the provided context and optional remaining accounts information.
    /// It handles the fee collection process of the protocol.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `CollectFees` instruction.
    /// * `remaining_accounts_info` - Optional information about remaining accounts, represented as `Option<RemainingAccountsInfo>`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the fees are successfully collected,
    /// or an error if it fails.
    pub fn collect_fees<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectFees<'info>>,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::collect_fees::collect_fees_handler(ctx, remaining_accounts_info);
    }

    /// Collects protocol fees for ai dex of the protocol.
    ///
    /// This function collects protocol fees using the provided context and optional remaining accounts information.
    /// It handles the fee collection process of the protocol.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `CollectProtocolFees` instruction.
    /// * `remaining_accounts_info` - Optional information about remaining accounts, represented as `Option<RemainingAccountsInfo>`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the protocol fees are successfully collected,
    /// or an error if it fails.
    pub fn collect_protocol_fees<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectProtocolFees<'info>>,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::collect_protocol_fees::collect_protocol_fees_handler(ctx, remaining_accounts_info);
    }

    /// Collects rewards for the position.
    ///
    /// This function collects rewards using the provided context, reward index, and optional remaining accounts information.
    /// It handles the reward collection process of the protocol.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `CollectReward` instruction.
    /// * `reward_index` - The index of the reward to collect, represented as a `u8`.
    /// * `remaining_accounts_info` - Optional information about remaining accounts, represented as `Option<RemainingAccountsInfo>`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the rewards are successfully collected,
    /// or an error if it fails.
    pub fn collect_reward<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectReward<'info>>,
        reward_index: u8,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::collect_reward::collect_reward_handler(ctx, reward_index, remaining_accounts_info);
    }

    /// Decreases the liquidity for a position in the ai dex pool with additional account information.
    ///
    /// This function reduces the liquidity for the specified position, ensuring that the minimum
    /// token amounts are met. It uses the provided context and optional remaining accounts information
    /// to perform the operation.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `ModifyLiquidity` instruction.
    /// * `liquidity_amount` - The amount of liquidity to be decreased, represented as a `u128`.
    /// * `token_min_a` - The minimum amount of token A to be received, represented as a `u64`.
    /// * `token_min_b` - The minimum amount of token B to be received, represented as a `u64`.
    /// * `remaining_accounts_info` - Optional additional account information for the operation.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the liquidity decrease is successful,
    /// or an error if it fails.
    pub fn decrease_liquidity<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ModifyLiquidity<'info>>,
        liquidity_amount: u128,
        token_min_a: u64,
        token_min_b: u64,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::decrease_liquidity::decrease_liquidity_handler(
            ctx,
            liquidity_amount,
            token_min_a,
            token_min_b,
            remaining_accounts_info,
        );
    }

    /// Increases the liquidity for a position in the ai dex pool with additional parameters.
    ///
    /// This function increases the liquidity for a position using the specified amounts of tokens.
    /// It also allows for additional account information to be provided.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `ModifyLiquidity` instruction.
    /// * `liquidity_amount` - The amount of liquidity to add, represented as a `u128`.
    /// * `token_max_a` - The maximum amount of token A to use, represented as a `u64`.
    /// * `token_max_b` - The maximum amount of token B to use, represented as a `u64`.
    /// * `remaining_accounts_info` - Optional additional account information.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the liquidity increase is successful,
    /// or an error if it fails.
    pub fn increase_liquidity<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ModifyLiquidity<'info>>,
        liquidity_amount: u128,
        token_max_a: u64,
        token_max_b: u64,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::increase_liquidity::increase_liquidity_handler(
            ctx,
            liquidity_amount,
            token_max_a,
            token_max_b,
            remaining_accounts_info,
        );
    }

    /// Initializes a new ai dex pool with the given parameters.
    ///
    /// This function sets up a new pool with the specified tick spacing and initial square root price.
    /// It uses the provided context to initialize the pool.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializePool` instruction.
    /// * `tick_spacing` - The spacing between ticks in the pool, represented as a `u16`.
    /// * `initial_sqrt_price` - The initial square root price of the pool, represented as a `u128`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the pool initialization is successful,
    /// or an error if it fails.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        tick_spacing: u16,
        initial_sqrt_price: u128,
    ) -> Result<()> {
        return instructions::initialize_pool::initialize_pool_handler(
            ctx,
            tick_spacing,
            initial_sqrt_price,
        );
    }

    /// Initializes a new reward for an ai dex. 
    ///
    /// A pool can only support up to a set number of rewards.
    ///
    /// This function sets up a new reward for the specified pool. It uses the provided context
    /// and reward index to initialize the reward.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializeReward` instruction.
    /// * `reward_index` - The index of the reward to be initialized, represented as a `u8`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the reward initialization is successful,
    /// or an error if it fails.
    pub fn initialize_reward(ctx: Context<InitializeReward>, reward_index: u8) -> Result<()> {
        return instructions::initialize_reward::initialize_reward_handler(ctx, reward_index);
    }

    /// Sets the reward emissions rate for a specific reward in the ai dex pool (version 2).
    ///
    /// This function updates the emissions rate for the specified reward index in the pool.
    /// It uses the provided context to set the emissions rate.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `SetRewardEmissions` instruction.
    /// * `reward_index` - The index of the reward to update, represented as a `u8`.
    /// * `emissions_per_second_x64` - The emissions rate per second for the reward, represented as a `u128`.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the emissions rate is successfully set,
    /// or an error if it fails.
    pub fn set_reward_emissions(
        ctx: Context<SetRewardEmissions>,
        reward_index: u8,
        emissions_per_second_x64: u128,
    ) -> Result<()> {
        return instructions::set_reward_emissions::set_reward_emissions_handler(
            ctx,
            reward_index,
            emissions_per_second_x64,
        );
    }

    /// Executes a swap operation in the AI DEX protocol.
    ///
    /// This function performs a swap operation with the specified parameters. It uses the provided context
    /// and additional parameters to execute the swap.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `Swap` instruction.
    /// * `amount` - The amount to be swapped, represented as a `u64`.
    /// * `other_amount_threshold` - The threshold for the other amount in the swap, represented as a `u64`.
    /// * `sqrt_price_limit` - The square root price limit for the swap, represented as a `u128`.
    /// * `amount_specified_is_input` - A boolean indicating whether the specified amount is the input amount.
    /// * `a_to_b` - A boolean indicating the direction of the swap (true for A to B, false for B to A).
    /// * `remaining_accounts_info` - Optional remaining accounts information for the swap.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the swap is successful, or an error if it fails.
    pub fn swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Swap<'info>>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::swap::swap_handler(
            ctx,
            amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
            remaining_accounts_info,
        );
    }

    /// Executes a two-hop swap with the given parameters.
    ///
    /// This function performs a two-hop swap operation, which involves swapping tokens
    /// through two different pools. It uses the provided context and parameters to execute
    /// the swap.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `TwoHopSwap` instruction.
    /// * `amount` - The amount of tokens to swap.
    /// * `other_amount_threshold` - The threshold for the other amount in the swap.
    /// * `amount_specified_is_input` - A boolean indicating if the specified amount is the input amount.
    /// * `a_to_b_one` - A boolean indicating the direction of the first swap (A to B).
    /// * `a_to_b_two` - A boolean indicating the direction of the second swap (A to B).
    /// * `sqrt_price_limit_one` - The square root price limit for the first swap.
    /// * `sqrt_price_limit_two` - The square root price limit for the second swap.
    /// * `remaining_accounts_info` - Optional remaining accounts information.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the swap is successful,
    /// or an error if it fails.
    pub fn two_hop_swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, TwoHopSwap<'info>>,
        amount: u64,
        other_amount_threshold: u64,
        amount_specified_is_input: bool,
        a_to_b_one: bool,
        a_to_b_two: bool,
        sqrt_price_limit_one: u128,
        sqrt_price_limit_two: u128,
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    ) -> Result<()> {
        return instructions::two_hop_swap::two_hop_swap_handler(
            ctx,
            amount,
            other_amount_threshold,
            amount_specified_is_input,
            a_to_b_one,
            a_to_b_two,
            sqrt_price_limit_one,
            sqrt_price_limit_two,
            remaining_accounts_info,
        );
    }

    /// Initializes the token wrapper for the AI DEX protocol.
    ///
    /// This function sets up the token wrapper using the provided context.
    /// It initializes the necessary parameters and settings for the token wrapper.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `InitializeTokenWrapper` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the token wrapper is successfully initialized,
    /// or an error if it fails.
    pub fn initialize_token_wrapper(ctx: Context<InitializeTokenWrapper>) -> Result<()> {
        return instructions::wrapper::initialize_token_wrapper::initialize_token_wrapper_handler(ctx);
    }

    /// Deletes the token wrapper in the AI DEX protocol.
    ///
    /// This function removes the token wrapper using the provided context.
    /// It performs the necessary operations to delete the token wrapper.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the `DeleteTokenWrapper` instruction.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is `Ok` if the token wrapper is successfully deleted,
    /// or an error if it fails.
    pub fn delete_token_wrapper(ctx: Context<DeleteTokenWrapper>) -> Result<()> {
        return instructions::wrapper::delete_token_wrapper::delete_token_wrapper_handler(ctx);
    }
}
