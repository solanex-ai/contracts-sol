use crate::state::{TokenWrapper, AiDexPool};
use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::{TransferFee, MAX_FEE_BASIS_POINTS};
use anchor_spl::token_interface::spl_token_2022::extension::BaseStateWithExtensions;

use anchor_spl::token::Token;
use anchor_spl::token_2022::spl_token_2022::{self, extension::{self, StateWithExtensions}, state::AccountState};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::memo::{self, Memo, BuildMemo};
use spl_transfer_hook_interface;

/// Transfers tokens from the owner's account to the vault.
///
/// This function performs the following steps:
/// 1. Checks for and logs any applicable transfer fees using the memo program.
/// 2. Creates a transfer instruction using the `spl_token_2022::instruction::transfer_checked` function.
/// 3. Prepares the necessary account infos for the transfer instruction.
/// 4. Handles any transfer hooks by adding extra accounts if required.
/// 5. Invokes the transfer instruction.
///
/// # Arguments
///
/// * `authority` - A reference to the signer authority.
/// * `token_mint` - A reference to the token mint account.
/// * `token_owner_account` - A reference to the token owner's token account.
/// * `token_vault` - A reference to the token vault account.
/// * `token_program` - A reference to the token program interface.
/// * `memo_program` - A reference to the memo program.
/// * `transfer_hook_accounts` - An optional vector of additional accounts for transfer hooks.
/// * `amount` - The amount of tokens to transfer.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the transfer is successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if there is an issue with logging the transfer fee, creating the transfer instruction,
/// preparing the account infos, handling the transfer hooks, or invoking the transfer instruction.
pub fn transfer_from_owner_to_vault<'info>(
    authority: &Signer<'info>,
    token_mint: &InterfaceAccount<'info, Mint>,
    token_owner_account: &InterfaceAccount<'info, TokenAccount>,
    token_vault: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    memo_program: &Program<'info, Memo>,
    transfer_hook_accounts: &Option<Vec<AccountInfo<'info>>>,
    amount: u64,
) -> Result<()> {
    // Handle TransferFee extension
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        // log applied transfer fee
        // - Not must, but important for ease of investigation and replay when problems occur
        // - Use Memo because logs risk being truncated
        let transfer_fee_memo = format!(
            "TFe: {}, {}",
            u16::from(epoch_transfer_fee.transfer_fee_basis_points),
            u64::from(epoch_transfer_fee.maximum_fee),
        );
        memo::build_memo(
            CpiContext::new(
                memo_program.to_account_info(),
                BuildMemo {}
            ),
            transfer_fee_memo.as_bytes()
        )?;
    }

    // Create transfer instruction
    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        &token_owner_account.key(), // from
        &token_mint.key(), // mint
        &token_vault.key(), // to
        authority.key, // authority
        &[],
        amount,
        token_mint.decimals,
    )?;

    // Prepare account infos
    let mut account_infos = vec![
        token_program.to_account_info(),
        token_owner_account.to_account_info(),
        token_mint.to_account_info(),
        token_vault.to_account_info(),
        authority.to_account_info(),
    ];

    // Handle TransferHook extension
    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        if let Some(hook_accounts) = transfer_hook_accounts {
            spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
                &mut instruction,
                &mut account_infos,
                &hook_program_id,
                token_owner_account.to_account_info(),
                token_mint.to_account_info(),
                token_vault.to_account_info(),
                authority.to_account_info(),
                amount,
                hook_accounts,
            )?;
        } else {
            return Err(ErrorCode::MissingExtraAccountsForTransferHookError.into());
        }
    }

    // Invoke the instruction
    solana_program::program::invoke_signed(
        &instruction,
        &account_infos,
        &[],
    )?;

    Ok(())
}

/// Builds and logs a memo using the provided memo program and content.
///
/// This function constructs a memo using the `memo::build_memo` function and logs it
/// using the provided memo program.
///
/// # Arguments
///
/// * `memo_program` - A reference to the memo program.
/// * `memo_content` - The content of the memo to be logged.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the memo is successfully built and logged, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if there is an issue with building or logging the memo.
fn build_and_log_memo<'info>(
    memo_program: &Program<'info, Memo>,
    memo_content: &[u8],
) -> Result<()> {
    memo::build_memo(
        CpiContext::new(
            memo_program.to_account_info(),
            BuildMemo {},
        ),
        memo_content,
    )
}

/// Transfers tokens from the vault to the owner's account.
///
/// This function performs the following steps:
/// 1. Checks for and logs any applicable transfer fees using the memo program.
/// 2. Logs a memo if required by the token owner's account.
/// 3. Creates a transfer instruction using the `spl_token_2022::instruction::transfer_checked` function.
/// 4. Prepares the necessary account infos for the transfer instruction.
/// 5. Handles any transfer hooks by adding extra accounts if required.
/// 6. Invokes the transfer instruction.
///
/// # Arguments
///
/// * `ai_dex` - A reference to the AiDex account.
/// * `token_mint` - A reference to the token mint account.
/// * `token_vault` - A reference to the token vault account.
/// * `token_owner_account` - A reference to the token owner's token account.
/// * `token_program` - A reference to the token program interface.
/// * `memo_program` - A reference to the memo program.
/// * `transfer_hook_accounts` - An optional vector of additional accounts for transfer hooks.
/// * `amount` - The amount of tokens to transfer.
/// * `memo` - The memo to be logged if required.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the transfer is successful, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if there is an issue with logging the transfer fee, logging the memo,
/// creating the transfer instruction, preparing the account infos, handling the transfer hooks,
/// or invoking the transfer instruction.
pub fn transfer_from_vault_to_owner<'info>(
    ai_dex: &Account<'info, AiDexPool>,
    token_mint: &InterfaceAccount<'info, Mint>,
    token_vault: &InterfaceAccount<'info, TokenAccount>,
    token_owner_account: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    memo_program: &Program<'info, Memo>,
    transfer_hook_accounts: &Option<Vec<AccountInfo<'info>>>,
    amount: u64,
    memo: &[u8],
) -> Result<()> {
    // Handle TransferFee extension
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        let transfer_fee_memo = format!(
            "TFe: {}, {}",
            u16::from(epoch_transfer_fee.transfer_fee_basis_points),
            u64::from(epoch_transfer_fee.maximum_fee),
        );
        build_and_log_memo(memo_program, transfer_fee_memo.as_bytes())?;
    }

    // Handle MemoTransfer extension
    if is_transfer_memo_required(&token_owner_account)? {
        build_and_log_memo(memo_program, memo)?;
    }

    // Create transfer instruction
    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        &token_vault.key(), // from
        &token_mint.key(), // mint
        &token_owner_account.key(), // to
        &ai_dex.key(), // authority
        &[],
        amount,
        token_mint.decimals,
    )?;

    // Prepare account infos
    let mut account_infos = vec![
        token_program.to_account_info(),
        token_vault.to_account_info(),
        token_mint.to_account_info(),
        token_owner_account.to_account_info(),
        ai_dex.to_account_info(),
    ];

    // Handle TransferHook extension
    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        if let Some(hook_accounts) = transfer_hook_accounts {
            spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
                &mut instruction,
                &mut account_infos,
                &hook_program_id,
                token_owner_account.to_account_info(),
                token_mint.to_account_info(),
                token_vault.to_account_info(),
                ai_dex.to_account_info(),
                amount,
                hook_accounts,
            )?;
        } else {
            return Err(ErrorCode::MissingExtraAccountsForTransferHookError.into());
        }
    }

    // Invoke the instruction
    solana_program::program::invoke_signed(
        &instruction,
        &account_infos,
        &[&ai_dex.seeds()],
    )?;

    Ok(())
}

/// Retrieves the transfer hook program ID for a given token mint.
///
/// This function checks if the token mint is owned by the Token Program and, if not,
/// retrieves the transfer hook program ID from the token mint's extensions.
///
/// # Arguments
///
/// * `token_mint` - A reference to the token mint account.
///
/// # Returns
///
/// * `Result<Option<Pubkey>>` - Returns `Ok(Some(Pubkey))` if a transfer hook program ID is found,
///   `Ok(None)` if the token mint is owned by the Token Program, otherwise returns an error.
///
/// # Errors
///
/// Returns an error if there is an issue with borrowing data or unpacking the mint data.
fn get_transfer_hook_program_id<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<Option<Pubkey>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    Ok(extension::transfer_hook::get_program_id(&token_mint_unpacked))
}

/// Checks if a transfer memo is required for a given token account.
///
/// This function checks if the token account is owned by the Token Program and, if not,
/// retrieves the memo transfer extension to determine if incoming transfer memos are required.
///
/// # Arguments
///
/// * `token_account` - A reference to the token account.
///
/// # Returns
///
/// * `Result<bool>` - Returns `Ok(true)` if incoming transfer memos are required,
///   `Ok(false)` if the token account is owned by the Token Program or if the memo transfer extension is not found.
///
/// # Errors
///
/// Returns an error if there is an issue with borrowing data or unpacking the account data.
fn is_transfer_memo_required<'info>(token_account: &InterfaceAccount<'info, TokenAccount>) -> Result<bool> {
    let token_account_info = token_account.to_account_info();
    if *token_account_info.owner == Token::id() {
        return Ok(false);
    }

    let token_account_data = token_account_info.try_borrow_data()?;
    let token_account_unpacked = StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account_data)?;
    let extension = token_account_unpacked.get_extension::<extension::memo_transfer::MemoTransfer>();

    if let Ok(memo_transfer) = extension {
        return Ok(memo_transfer.require_incoming_transfer_memos.into());
    } else {
        return Ok(false);
    }
}

/// Checks if the given token mint is supported.
///
/// This function performs several checks to determine if a token mint is supported:
/// 1. Checks if the mint is owned by the Token Program.
/// 2. Checks if the mint is the native mint of the Token-2022 Program.
/// 3. Checks if the mint has a freeze authority and if the token wrapper is initialized.
/// 4. Unpacks the mint data and iterates over the extension types to handle each case accordingly.
///
/// # Arguments
///
/// * `token_mint` - A reference to the token mint account.
/// * `is_token_wrapper_initialized` - A boolean indicating if the token wrapper is initialized.
///
/// # Returns
///
/// * `Result<bool>` - Returns `Ok(true)` if the token mint is supported, otherwise returns `Ok(false)`.
///
/// # Errors
///
/// Returns an error if there is an issue with borrowing data or unpacking the mint data.
pub fn is_supported_token_mint<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
    is_token_wrapper_initialized: bool,
) -> Result<bool> {
    let token_mint_info = token_mint.to_account_info();

    // Check if mint is owned by the Token Program
    if *token_mint_info.owner == Token::id() {
        return Ok(true);
    }

    // Check if mint is the native mint of the Token-2022 Program
    if spl_token_2022::native_mint::check_id(&token_mint.key()) {
        return Ok(false);
    }

    // Check if mint has a freeze authority and if the token wrapper is initialized
    if token_mint.freeze_authority.is_some() && !is_token_wrapper_initialized {
        return Ok(false);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    let extensions = token_mint_unpacked.get_extension_types()?;

    for extension in extensions {
        match extension {
            // supported
            extension::ExtensionType::TransferFeeConfig |
            extension::ExtensionType::TokenMetadata |
            extension::ExtensionType::MetadataPointer => {
                // Supported extensions
            }
            // Supported, but non-confidential transfer only
            //
            // AiDexProgram invokes TransferChecked instruction and it supports non-confidential transfer only.
            //
            // Because the vault accounts are not configured to support confidential transfer,
            // it is impossible to send tokens directly to the vault accounts confidentially.
            // Note: Only the owner (AiDex account) can call ConfidentialTransferInstruction::ConfigureAccount.
            extension::ExtensionType::ConfidentialTransferMint |
            
            extension::ExtensionType::ConfidentialTransferFeeConfig => {
                // Supported, but non-confidential transfer only
                // When both TransferFeeConfig and ConfidentialTransferMint are initialized,
                // ConfidentialTransferFeeConfig is also initialized to store encrypted transfer fee amount.
            }
            extension::ExtensionType::PermanentDelegate |
            extension::ExtensionType::TransferHook |
            extension::ExtensionType::MintCloseAuthority |
            extension::ExtensionType::DefaultAccountState => {
                if !is_token_wrapper_initialized {
                    return Ok(false);
                }

                // reject if default state is not Initialized even if it has token wrapper
                if let extension::ExtensionType::DefaultAccountState = extension {
                    let default_state = token_mint_unpacked.get_extension::<extension::default_account_state::DefaultAccountState>()?;
                    let initialized: u8 = AccountState::Initialized.into();
                    if default_state.state != initialized {
                        return Ok(false);
                    }
                }
            }
            // No possibility to support the following extensions
            extension::ExtensionType::NonTransferable => {
                return Ok(false);
            }
            // mint has unknown or unsupported extensions
            _ => {
                return Ok(false);
            }
        }
    }

    return Ok(true);
}

/// Checks if the token wrapper is initialized with the given configuration and mint keys.
///
/// # Arguments
///
/// * `ai_dex_config_key` - The public key of the AI DEX configuration.
/// * `token_mint_key` - The public key of the token mint.
/// * `token_wrapper` - The unchecked account of the token wrapper.
///
/// # Returns
///
/// * `Result<bool>` - Returns `true` if the token wrapper is initialized with the given keys, otherwise `false`.
pub fn is_token_wrapper_initialized<'info>(
    ai_dex_config_key: Pubkey,
    token_mint_key: Pubkey,
    token_wrapper: &UncheckedAccount<'info>,
) -> Result<bool> {
    // Check if the token wrapper account is owned by the expected program ID
    if *token_wrapper.owner != crate::id() {
        return Ok(false);
    }

    // Borrow the data from the token wrapper account
    let token_wrapper_data = token_wrapper.data.borrow();
    // Deserialize the borrowed data into a TokenWrapper object
    let token_wrapper = TokenWrapper::try_deserialize(&mut &token_wrapper_data[..])?;

    // Compare the ai_dex_config and token_mint fields with the provided keys
    Ok(
        token_wrapper.ai_dex_config == ai_dex_config_key &&
        token_wrapper.token_mint == token_mint_key
    )
}

#[derive(Debug)]
pub struct TransferFeeIncludedAmount {
    pub amount: u64,
    pub transfer_fee: u64,
}

#[derive(Debug)]
pub struct TransferFeeExcludedAmount {
    pub amount: u64,
    pub transfer_fee: u64,
}

pub fn calculate_transfer_fee_excluded_amount<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
    transfer_fee_included_amount: u64,
) -> Result<TransferFeeExcludedAmount> {
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        let transfer_fee = epoch_transfer_fee.calculate_fee(transfer_fee_included_amount).unwrap();
        let transfer_fee_excluded_amount = transfer_fee_included_amount.checked_sub(transfer_fee).unwrap();
        return Ok(TransferFeeExcludedAmount { amount: transfer_fee_excluded_amount, transfer_fee });            
    }

    Ok(TransferFeeExcludedAmount { amount: transfer_fee_included_amount, transfer_fee: 0 })
} 

pub fn calculate_transfer_fee_included_amount<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
    transfer_fee_excluded_amount: u64,
) -> Result<TransferFeeIncludedAmount> {
    if transfer_fee_excluded_amount == 0 {
        return Ok(TransferFeeIncludedAmount { amount: 0, transfer_fee: 0 });
    }

    // now transfer_fee_excluded_amount > 0

    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        let transfer_fee: u64 = if u16::from(epoch_transfer_fee.transfer_fee_basis_points) == MAX_FEE_BASIS_POINTS {
            // edge-case: if transfer fee rate is 100%, current SPL implementation returns 0 as inverse fee.
            // https://github.com/solana-labs/solana-program-library/blob/fe1ac9a2c4e5d85962b78c3fc6aaf028461e9026/token/program-2022/src/extension/transfer_fee/mod.rs#L95
            
            // But even if transfer fee is 100%, we can use maximum_fee as transfer fee.
            // if transfer_fee_excluded_amount + maximum_fee > u64 max, the following checked_add should fail.
            u64::from(epoch_transfer_fee.maximum_fee)
        } else {
            epoch_transfer_fee.calculate_inverse_fee(transfer_fee_excluded_amount)
                .ok_or(ErrorCode::TransferFeeCalculationError)?
        };

        let transfer_fee_included_amount = transfer_fee_excluded_amount.checked_add(transfer_fee)
            .ok_or(ErrorCode::TransferFeeCalculationError)?;

        // verify transfer fee calculation for safety
        let transfer_fee_verification = epoch_transfer_fee.calculate_fee(transfer_fee_included_amount).unwrap();
        if transfer_fee != transfer_fee_verification {
            // We believe this should never happen
            return Err(ErrorCode::TransferFeeCalculationError.into());
        }

        return Ok(TransferFeeIncludedAmount { amount: transfer_fee_included_amount, transfer_fee });
    }

    Ok(TransferFeeIncludedAmount { amount: transfer_fee_excluded_amount, transfer_fee: 0 })
}

pub fn get_epoch_transfer_fee<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<Option<TransferFee>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    if let Ok(transfer_fee_config) = token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>() {
        let epoch = Clock::get()?.epoch;
        return Ok(Some(transfer_fee_config.get_epoch_fee(epoch).clone()));
    }

    Ok(None)
}

#[cfg(test)]
mod fuzz_tests {
    use proptest::prelude::*;
    use super::*;

    struct SyscallStubs {}
    impl solana_program::program_stubs::SyscallStubs for SyscallStubs {
        fn sol_get_clock_sysvar(&self, _var_addr: *mut u8) -> u64 {
            0
        }
    }

    #[derive(Default, AnchorSerialize)]
    struct MintWithTransferFeeConfigLayout {
        // 82 for Mint
        pub coption_mint_authority: u32, // 4
        pub mint_authority: Pubkey, // 32
        pub supply: u64, // 8
        pub decimals: u8, // 1
        pub is_initialized: bool, // 1
        pub coption_freeze_authority: u32, // 4
        pub freeze_authority: Pubkey, // 4 + 32

        // 83 for padding
        pub padding1: [u8; 32],
        pub padding2: [u8; 32],
        pub padding3: [u8; 19],

        pub account_type: u8, // 1

        pub extension_type: u16, // 2
        pub extension_length: u16, // 2
        // 108 for TransferFeeConfig data
        pub transfer_fee_config_authority: Pubkey, // 32
        pub withdraw_withheld_authority: Pubkey, // 32
        pub withheld_amount: u64, // 8
        pub older_epoch: u64, // 8
        pub older_maximum_fee: u64, // 8
        pub older_transfer_fee_basis_point: u16, // 2
        pub newer_epoch: u64, // 8
        pub newer_maximum_fee: u64, // 8
        pub newer_transfer_fee_basis_point: u16, // 2
    }
    impl MintWithTransferFeeConfigLayout {
        pub const LEN: usize = 82 + 83 + 1 + 2 + 2 + 108;
    }

    /// Maximum possible fee in basis points is 100%, aka 10_000 basis points
    const MAX_FEE_BASIS_POINTS: u16 = 10_000;
    const MAX_FEE: u64 = 1_000_000_000;
    const MAX_AMOUNT: u64 = 0xFFFFFFFF;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100000))]
        #[test]
        fn test_calculate_transfer_fee_included_amount(
            amount in 0..MAX_AMOUNT,
            maximum_fee in 0..MAX_FEE,
            transfer_fee_basis_point in 0..MAX_FEE_BASIS_POINTS
        ) {
            // stub Clock
            solana_program::program_stubs::set_syscall_stubs(Box::new(SyscallStubs {}));
            assert_eq!(Clock::get().unwrap().epoch, 0);

            let mint_with_transfer_fee_config = MintWithTransferFeeConfigLayout {
                is_initialized: true,
                account_type: 1, // Mint
                extension_type: 1, // TransferFeeConfig
                extension_length: 108,
                older_epoch: 0,
                older_maximum_fee: maximum_fee,
                older_transfer_fee_basis_point: transfer_fee_basis_point,
                newer_epoch: 0,
                newer_maximum_fee: maximum_fee,
                newer_transfer_fee_basis_point: transfer_fee_basis_point,
                ..Default::default()
            };

            let mut data = Vec::<u8>::new();
            mint_with_transfer_fee_config.serialize(&mut data).unwrap();
            assert_eq!(data.len(), MintWithTransferFeeConfigLayout::LEN);

            let key = Pubkey::default();
            let mut lamports = 0u64;
            let owner = anchor_spl::token_2022::ID;
            let rent_epoch = 0;
            let is_signer = false;
            let is_writable = false;
            let executable = false;
            let account_info = AccountInfo::new(
                &key,
                is_signer,
                is_writable,
                &mut lamports,
                &mut data,
                &owner,
                executable,
                rent_epoch
            );
    
            let interface_account_mint = InterfaceAccount::<Mint>::try_from(&account_info).unwrap();

            let transfer_fee = get_epoch_transfer_fee(&interface_account_mint).unwrap().unwrap();
            assert_eq!(u64::from(transfer_fee.maximum_fee), maximum_fee);
            assert_eq!(u16::from(transfer_fee.transfer_fee_basis_points), transfer_fee_basis_point);

            let _ = calculate_transfer_fee_included_amount(&interface_account_mint, amount)?;
        }
    }
}