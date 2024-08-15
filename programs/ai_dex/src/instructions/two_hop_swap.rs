use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::memo::Memo;

use crate::swap_with_transfer_fee_extension;
use crate::util::{calculate_transfer_fee_excluded_amount, parse_remaining_accounts, update_and_two_hop_swap_ai_dex, AccountsType, RemainingAccountsInfo};
use crate::{
    errors::ErrorCode,
    state::{TickArray, AiDexPool},
    util::{to_timestamp_u64, SwapTickSequence},
    constants::transfer_memo,
};

#[event]
pub struct TwoHopSwapEvent {
    pub ai_dex_one: Pubkey,
    pub ai_dex_two: Pubkey,
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub amount_specified_is_input: bool,
    pub a_to_b_one: bool,
    pub a_to_b_two: bool,
    pub sqrt_price_limit_one: u128,
    pub sqrt_price_limit_two: u128,
    pub timestamp: u64,
    pub token_mint_input: Pubkey,
    pub token_mint_intermediate: Pubkey,
    pub token_mint_output: Pubkey,
    pub token_program_input: Pubkey,
    pub token_program_intermediate: Pubkey,
    pub token_program_output: Pubkey,
    pub token_owner_account_input: Pubkey,
    pub token_vault_one_input: Pubkey,
    pub token_vault_one_intermediate: Pubkey,
    pub token_vault_two_intermediate: Pubkey,
    pub token_vault_two_output: Pubkey,
    pub token_owner_account_output: Pubkey,
    pub token_authority: Pubkey,
    pub tick_array_one_0: Pubkey,
    pub tick_array_one_1: Pubkey,
    pub tick_array_one_2: Pubkey,
    pub tick_array_two_0: Pubkey,
    pub tick_array_two_1: Pubkey,
    pub tick_array_two_2: Pubkey,
}

#[derive(Accounts)]
#[instruction(
    amount: u64,
    other_amount_threshold: u64,
    amount_specified_is_input: bool,
    a_to_b_one: bool,
    a_to_b_two: bool,
)]
/// Represents a two-hop swap operation involving two different AiDex instances.
pub struct TwoHopSwap<'info> {
    /// The first AiDex instance involved in the swap.
    #[account(mut)]
    pub ai_dex_one: Box<Account<'info, AiDexPool>>,
    
    /// The second AiDex instance involved in the swap.
    #[account(mut)]
    pub ai_dex_two: Box<Account<'info, AiDexPool>>,

    /// The mint account for the input token.
    #[account(address = ai_dex_one.input_token_mint(a_to_b_one))]
    pub token_mint_input: InterfaceAccount<'info, Mint>,
    
    /// The mint account for the intermediate token.
    #[account(address = ai_dex_one.output_token_mint(a_to_b_one))]
    pub token_mint_intermediate: InterfaceAccount<'info, Mint>,
    
    /// The mint account for the output token.
    #[account(address = ai_dex_two.output_token_mint(a_to_b_two))]
    pub token_mint_output: InterfaceAccount<'info, Mint>,

    /// The token program for the input token.
    #[account(address = token_mint_input.to_account_info().owner.clone())]
    pub token_program_input: Interface<'info, TokenInterface>,
    
    /// The token program for the intermediate token.
    #[account(address = token_mint_intermediate.to_account_info().owner.clone())]
    pub token_program_intermediate: Interface<'info, TokenInterface>,
    
    /// The token program for the output token.
    #[account(address = token_mint_output.to_account_info().owner.clone())]
    pub token_program_output: Interface<'info, TokenInterface>,

    /// The token account of the owner for the input token.
    #[account(mut, constraint = token_owner_account_input.mint == token_mint_input.key())]
    pub token_owner_account_input: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// The token vault for the input token in the first AiDex.
    #[account(mut, address = ai_dex_one.input_token_vault(a_to_b_one))]
    pub token_vault_one_input: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// The token vault for the intermediate token in the first AiDex.
    #[account(mut, address = ai_dex_one.output_token_vault(a_to_b_one))]
    pub token_vault_one_intermediate: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The token vault for the intermediate token in the second AiDex.
    #[account(mut, address = ai_dex_two.input_token_vault(a_to_b_two))]
    pub token_vault_two_intermediate: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// The token vault for the output token in the second AiDex.
    #[account(mut, address = ai_dex_two.output_token_vault(a_to_b_two))]
    pub token_vault_two_output: Box<InterfaceAccount<'info, TokenAccount>>,
    
    /// The token account of the owner for the output token.
    #[account(mut, constraint = token_owner_account_output.mint == token_mint_output.key())]
    pub token_owner_account_output: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The authority that signs the transaction.
    pub token_authority: Signer<'info>,

    /// The first tick array for the first AiDex.
    #[account(mut, constraint = tick_array_one_0.load()?.ai_dex_pool == ai_dex_one.key())]
    pub tick_array_one_0: AccountLoader<'info, TickArray>,

    /// The second tick array for the first AiDex.
    #[account(mut, constraint = tick_array_one_1.load()?.ai_dex_pool == ai_dex_one.key())]
    pub tick_array_one_1: AccountLoader<'info, TickArray>,

    /// The third tick array for the first AiDex.
    #[account(mut, constraint = tick_array_one_2.load()?.ai_dex_pool == ai_dex_one.key())]
    pub tick_array_one_2: AccountLoader<'info, TickArray>,

    /// The first tick array for the second AiDex.
    #[account(mut, constraint = tick_array_two_0.load()?.ai_dex_pool == ai_dex_two.key())]
    pub tick_array_two_0: AccountLoader<'info, TickArray>,

    /// The second tick array for the second AiDex.
    #[account(mut, constraint = tick_array_two_1.load()?.ai_dex_pool == ai_dex_two.key())]
    pub tick_array_two_1: AccountLoader<'info, TickArray>,

    /// The third tick array for the second AiDex.
    #[account(mut, constraint = tick_array_two_2.load()?.ai_dex_pool == ai_dex_two.key())]
    pub tick_array_two_2: AccountLoader<'info, TickArray>,

    /// CHECK: The oracle account for the first AiDex (is currently unused).
    #[account(mut, seeds = [b"oracle", ai_dex_one.key().as_ref()], bump)]
    pub oracle_one: UncheckedAccount<'info>,

    /// CHECK: The oracle account for the second AiDex (is currently unused).
    #[account(mut, seeds = [b"oracle", ai_dex_two.key().as_ref()], bump)]
    pub oracle_two: UncheckedAccount<'info>,

    /// The memo program.
    pub memo_program: Program<'info, Memo>,

    // Remaining accounts:
    // - Accounts for transfer hook program of token_mint_input
    // - Accounts for transfer hook program of token_mint_intermediate
    // - Accounts for transfer hook program of token_mint_output
}

/// Handles a two-hop swap operation with specified parameters.
///
/// This function performs a two-hop swap, which involves two separate swap operations
/// between three tokens. It ensures that the intermediary token between the two swaps
/// matches and that the output of the first swap is used as the input for the second swap.
///
/// # Arguments
///
/// * `ctx` - The context containing all the accounts and programs required for the swap.
/// * `amount` - The amount to be swapped.
/// * `other_amount_threshold` - The minimum or maximum amount threshold for the swap.
/// * `amount_specified_is_input` - A boolean indicating if the specified amount is the input amount.
/// * `a_to_b_one` - A boolean indicating the direction of the first swap (A to B if true, B to A if false).
/// * `a_to_b_two` - A boolean indicating the direction of the second swap (A to B if true, B to A if false).
/// * `sqrt_price_limit_one` - The square root price limit for the first swap.
/// * `sqrt_price_limit_two` - The square root price limit for the second swap.
/// * `remaining_accounts_info` - Optional information about remaining accounts.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the swap is successful, or an `Err` if an error occurs.
///
/// # Errors
///
/// This function can return errors in the following cases:
/// * Duplicate two-hop pool error if the same pool is used for both swaps.
/// * Invalid intermediary mint error if the intermediary token does not match.
/// * Amount mismatch error if the output of the first swap does not match the input of the second swap.
/// * Amount out below minimum error if the output amount is less than the specified threshold.
/// * Amount in above maximum error if the input amount is more than the specified threshold.
pub fn two_hop_swap_handler<'a, 'b, 'c, 'info>(
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
    let clock = Clock::get()?;
    // Update the global reward growth which increases as a function of time.
    let timestamp = to_timestamp_u64(clock.unix_timestamp)?;

    let ai_dex_one = &mut ctx.accounts.ai_dex_one;
    let ai_dex_two = &mut ctx.accounts.ai_dex_two;
    // Don't allow swaps on the same ai_dex
    if ai_dex_one.key() == ai_dex_two.key() {
        return Err(ErrorCode::DuplicateTwoHopPoolError.into());
    }

    let swap_one_output_mint = match a_to_b_one {
        true => ai_dex_one.token_mint_b,
        false => ai_dex_one.token_mint_a,
    };

    let swap_two_input_mint = match a_to_b_two {
        true => ai_dex_two.token_mint_a,
        false => ai_dex_two.token_mint_b,
    };

    if swap_one_output_mint != swap_two_input_mint {
        return Err(ErrorCode::InvalidIntermediaryMintError.into());
    }
    // Process remaining accounts
    let remaining_accounts = parse_remaining_accounts(
        &ctx.remaining_accounts,
        &remaining_accounts_info,
        &[
            AccountsType::TransferHookInput,
            AccountsType::TransferHookIntermediate,
            AccountsType::TransferHookOutput,
        ],
    )?;

    let mut swap_tick_sequence_one = SwapTickSequence::new(
        ctx.accounts.tick_array_one_0.load_mut().unwrap(),
        ctx.accounts.tick_array_one_1.load_mut().ok(),
        ctx.accounts.tick_array_one_2.load_mut().ok(),
    );

    let mut swap_tick_sequence_two = SwapTickSequence::new(
        ctx.accounts.tick_array_two_0.load_mut().unwrap(),
        ctx.accounts.tick_array_two_1.load_mut().ok(),
        ctx.accounts.tick_array_two_2.load_mut().ok(),
    );
    // TODO: WLOG, we could extend this to N-swaps, but the account inputs to the instruction would
    // need to be jankier and we may need to programatically map/verify rather than using anchor constraints
    let (swap_update_one, swap_update_two) = match amount_specified_is_input {
        true => {
            // If the amount specified is input, this means we are doing exact-in
            // and the swap calculations occur from Swap 1 => Swap 2
            // and the swaps occur from Swap 1 => Swap 2
            let swap_calc_one = swap_with_transfer_fee_extension(
                &ai_dex_one,
                if a_to_b_one { &ctx.accounts.token_mint_input } else { &ctx.accounts.token_mint_intermediate },
                if a_to_b_one { &ctx.accounts.token_mint_intermediate } else { &ctx.accounts.token_mint_input },
                &mut swap_tick_sequence_one,
                amount,
                sqrt_price_limit_one,
                true,
                a_to_b_one,
                timestamp,
            )?;
            // Swap two input is the output of swap one
            // We use vault to vault transfer, so transfer fee will be collected once.
            let swap_two_input_amount = match a_to_b_one {
                true => swap_calc_one.amount_b,
                false => swap_calc_one.amount_a,
            };

            let swap_calc_two = swap_with_transfer_fee_extension(
                &ai_dex_two,
                if a_to_b_two { &ctx.accounts.token_mint_intermediate } else { &ctx.accounts.token_mint_output },
                if a_to_b_two { &ctx.accounts.token_mint_output } else { &ctx.accounts.token_mint_intermediate },
                &mut swap_tick_sequence_two,
                swap_two_input_amount,
                sqrt_price_limit_two,
                true,
                a_to_b_two,
                timestamp,
            )?;
            (swap_calc_one, swap_calc_two)
        },
        false => {
            // If the amount specified is output, this means we need to invert the ordering of the calculations
            // and the swap calculations occur from Swap 2 => Swap 1
            // but the actual swaps occur from Swap 1 => Swap 2 (to ensure that the intermediate token exists in the account)
            let swap_calc_two = swap_with_transfer_fee_extension(
                &ai_dex_two,
                if a_to_b_two { &ctx.accounts.token_mint_intermediate } else { &ctx.accounts.token_mint_output },
                if a_to_b_two { &ctx.accounts.token_mint_output } else { &ctx.accounts.token_mint_intermediate },
                &mut swap_tick_sequence_two,
                amount,
                sqrt_price_limit_two,
                false,
                a_to_b_two,
                timestamp,
            )?;
            // The output of swap 1 is input of swap_calc_two
            let swap_one_output_amount = match a_to_b_two {
                true => calculate_transfer_fee_excluded_amount(
                    &ctx.accounts.token_mint_intermediate,
                    swap_calc_two.amount_a
                )?.amount,
                false => calculate_transfer_fee_excluded_amount(
                    &ctx.accounts.token_mint_intermediate,
                    swap_calc_two.amount_b
                )?.amount,
            };

            let swap_calc_one = swap_with_transfer_fee_extension(
                &ai_dex_one,
                if a_to_b_one { &ctx.accounts.token_mint_input } else { &ctx.accounts.token_mint_intermediate },
                if a_to_b_one { &ctx.accounts.token_mint_intermediate } else { &ctx.accounts.token_mint_input },
                &mut swap_tick_sequence_one,
                swap_one_output_amount,
                sqrt_price_limit_one,
                false,
                a_to_b_one,
                timestamp,
            )?;
            (swap_calc_one, swap_calc_two)
        },
    };
    // All output token should be consumed by the second swap
    let swap_calc_one_output = match a_to_b_one {
        true => swap_update_one.amount_b,
        false => swap_update_one.amount_a,
    };
    let swap_calc_two_input = match a_to_b_two {
        true => swap_update_two.amount_a,
        false => swap_update_two.amount_b,
    };

    if swap_calc_one_output != swap_calc_two_input {
        return Err(ErrorCode::AmountMismatchError.into());
    }

    // If amount_specified_is_input == true, then we have a variable amount of output
    // The slippage we care about is the output of the second swap.
    if amount_specified_is_input {
        let output_amount = match a_to_b_two {
            true => calculate_transfer_fee_excluded_amount(
                &ctx.accounts.token_mint_output,
                swap_update_two.amount_b
            )?.amount,
            false => calculate_transfer_fee_excluded_amount(
                &ctx.accounts.token_mint_output,
                swap_update_two.amount_a
            )?.amount,
        };

        // If we have received less than the minimum out, throw an error
        if output_amount < other_amount_threshold {
            return Err(ErrorCode::AmountOutBelowMinimumError.into());
        }
    } else {
        // amount_specified_is_output == false, then we have a variable amount of input
        // The slippage we care about is the input of the first swap
        let input_amount = match a_to_b_one {
            true => swap_update_one.amount_a,
            false => swap_update_one.amount_b,
        };
        if input_amount > other_amount_threshold {
            return Err(ErrorCode::AmountInAboveMaximumError.into());
        }
    }

    update_and_two_hop_swap_ai_dex(
        swap_update_one,
        swap_update_two,
        ai_dex_one,
        ai_dex_two,
        a_to_b_one,
        a_to_b_two,
        &ctx.accounts.token_mint_input,
        &ctx.accounts.token_mint_intermediate,
        &ctx.accounts.token_mint_output,
        &ctx.accounts.token_program_input,
        &ctx.accounts.token_program_intermediate,
        &ctx.accounts.token_program_output,
        &ctx.accounts.token_owner_account_input,
        &ctx.accounts.token_vault_one_input,
        &ctx.accounts.token_vault_one_intermediate,
        &ctx.accounts.token_vault_two_intermediate,
        &ctx.accounts.token_vault_two_output,
        &ctx.accounts.token_owner_account_output,
        &remaining_accounts.transfer_hook_input,
        &remaining_accounts.transfer_hook_intermediate,
        &remaining_accounts.transfer_hook_output,
        &ctx.accounts.token_authority,
        &ctx.accounts.memo_program,
        timestamp,
        transfer_memo::TRANSFER_MEMO_SWAP.as_bytes(),
    )?;

    emit!(TwoHopSwapEvent {
        ai_dex_one: ai_dex_one.key(),
        ai_dex_two: ai_dex_two.key(),
        amount,
        other_amount_threshold,
        amount_specified_is_input,
        a_to_b_one,
        a_to_b_two,
        sqrt_price_limit_one,
        sqrt_price_limit_two,
        timestamp,
        token_mint_input: ctx.accounts.token_mint_input.key(),
        token_mint_intermediate: ctx.accounts.token_mint_intermediate.key(),
        token_mint_output: ctx.accounts.token_mint_output.key(),
        token_program_input: ctx.accounts.token_program_input.key(),
        token_program_intermediate: ctx.accounts.token_program_intermediate.key(),
        token_program_output: ctx.accounts.token_program_output.key(),
        token_owner_account_input: ctx.accounts.token_owner_account_input.key(),
        token_vault_one_input: ctx.accounts.token_vault_one_input.key(),
        token_vault_one_intermediate: ctx.accounts.token_vault_one_intermediate.key(),
        token_vault_two_intermediate: ctx.accounts.token_vault_two_intermediate.key(),
        token_vault_two_output: ctx.accounts.token_vault_two_output.key(),
        token_owner_account_output: ctx.accounts.token_owner_account_output.key(),
        token_authority: ctx.accounts.token_authority.key(),
        tick_array_one_0: ctx.accounts.tick_array_one_0.key(),
        tick_array_one_1: ctx.accounts.tick_array_one_1.key(),
        tick_array_one_2: ctx.accounts.tick_array_one_2.key(),
        tick_array_two_0: ctx.accounts.tick_array_two_0.key(),
        tick_array_two_1: ctx.accounts.tick_array_two_1.key(),
        tick_array_two_2: ctx.accounts.tick_array_two_2.key(),
    });

    Ok(())
}
