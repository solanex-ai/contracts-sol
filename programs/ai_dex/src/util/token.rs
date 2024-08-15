use crate::state::{PositionTradeBatch, AiDexPool};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::metadata::{self, CreateMetadataAccountsV3, mpl_token_metadata::types::DataV2};
use solana_program::program::invoke_signed;
use spl_token::instruction::{burn_checked, close_account, mint_to, set_authority, AuthorityType};

use crate::constants::nft::{
    ADB_METADATA_SYMBOL, ADB_METADATA_URI, AD_METADATA_NAME,
    AD_METADATA_SYMBOL, AD_METADATA_URI,
};

/// Burns a single token from the user's position token account and closes the account.
///
/// # Arguments
///
/// * `token_authority` - The signer authority for the token.
/// * `receiver` - The account to receive the remaining funds.
/// * `position_mint` - The mint of the position token.
/// * `position_token_account` - The user's position token account.
/// * `token_program` - The token program.
///
/// # Errors
///
/// Returns an error if the burn or close account operations fail.
pub fn burn_and_close_user_position_token<'info>(
    token_authority: &Signer<'info>,
    receiver: &UncheckedAccount<'info>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    // Burn a single token in user account
    invoke_signed(
        &burn_checked(
            token_program.key,
            position_token_account.to_account_info().key,
            position_mint.to_account_info().key,
            token_authority.key,
            &[],
            1,
            position_mint.decimals,
        )?,
        &[
            token_program.to_account_info(),
            position_token_account.to_account_info(),
            position_mint.to_account_info(),
            token_authority.to_account_info(),
        ],
        &[],
    )?;

    // Close user account
    invoke_signed(
        &close_account(
            token_program.key,
            position_token_account.to_account_info().key,
            receiver.key,
            token_authority.key,
            &[],
        )?,
        &[
            token_program.to_account_info(),
            position_token_account.to_account_info(),
            receiver.to_account_info(),
            token_authority.to_account_info(),
        ],
        &[],
    )?;
    Ok(())
}


/// Mints a position token and removes the mint authority.
///
/// # Arguments
///
/// * `ai_dex` - The AiDex account.
/// * `position_mint` - The mint of the position token.
/// * `position_token_account` - The position token account.
/// * `token_program` - The token program.
///
/// # Errors
///
/// Returns an error if the mint or authority removal fails.
pub fn mint_position_token_and_remove_authority<'info>(
    ai_dex: &Account<'info, AiDexPool>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    mint_position_token(
        ai_dex,
        position_mint,
        position_token_account,
        token_program,
    )?;
    remove_position_token_mint_authority(ai_dex, position_mint, token_program)
}

/// Mints a position token with metadata and removes the mint authority.
///
/// # Arguments
///
/// * `ai_dex` - The AiDex account.
/// * `position_mint` - The mint of the position token.
/// * `position_token_account` - The position token account.
/// * `position_metadata_account` - The position metadata account.
/// * `metadata_update_auth` - The metadata update authority.
/// * `funder` - The funder of the metadata account.
/// * `metadata_program` - The metadata program.
/// * `token_program` - The token program.
/// * `system_program` - The system program.
/// * `rent` - The rent sysvar.
///
/// # Errors
///
/// Returns an error if the mint, metadata creation, or authority removal fails.
pub fn mint_position_token_with_metadata_and_remove_authority<'info>(
    ai_dex: &Account<'info, AiDexPool>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    position_metadata_account: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    funder: &Signer<'info>,
    metadata_program: &Program<'info, metadata::Metadata>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    mint_position_token(
        ai_dex,
        position_mint,
        position_token_account,
        token_program,
    )?;

    let metadata_mint_auth_account = ai_dex;
    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: position_metadata_account.to_account_info(),
                mint: position_mint.to_account_info(),
                mint_authority: metadata_mint_auth_account.to_account_info(),
                update_authority: metadata_update_auth.to_account_info(),
                payer: funder.to_account_info(),
                rent: rent.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&metadata_mint_auth_account.seeds()],
        ),
        DataV2 {
            name: AD_METADATA_NAME.to_string(),
            symbol: AD_METADATA_SYMBOL.to_string(),
            uri: AD_METADATA_URI.to_string(),
            creators: None,
            seller_fee_basis_points: 0,
            collection: None,
            uses: None,
        },
        true,
        false,
        None,
    )?;

    remove_position_token_mint_authority(ai_dex, position_mint, token_program)
}

/// Mints a single position token to the specified token account.
///
/// # Arguments
///
/// * `ai_dex` - The AiDex account which has the authority to mint the token.
/// * `position_mint` - The mint of the position token.
/// * `position_token_account` - The account to receive the minted token.
/// * `token_program` - The token program.
///
/// # Errors
///
/// Returns an error if the mint operation fails.
fn mint_position_token<'info>(
    ai_dex: &Account<'info, AiDexPool>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    invoke_signed(
        &mint_to(
            token_program.key,
            position_mint.to_account_info().key,
            position_token_account.to_account_info().key,
            ai_dex.to_account_info().key,
            &[ai_dex.to_account_info().key],
            1,
        )?,
        &[
            position_mint.to_account_info(),
            position_token_account.to_account_info(),
            ai_dex.to_account_info(),
            token_program.to_account_info(),
        ],
        &[&ai_dex.seeds()],
    )?;
    Ok(())
}

/// Removes the mint authority from the position token.
///
/// # Arguments
///
/// * `ai_dex` - The AiDex account.
/// * `position_mint` - The mint of the position token.
/// * `token_program` - The token program.
///
/// # Errors
///
/// Returns an error if the authority removal fails.
fn remove_position_token_mint_authority<'info>(
    ai_dex: &Account<'info, AiDexPool>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    invoke_signed(
        &set_authority(
            token_program.key,
            position_mint.to_account_info().key,
            Option::None,
            AuthorityType::MintTokens,
            ai_dex.to_account_info().key,
            &[ai_dex.to_account_info().key],
        )?,
        &[
            position_mint.to_account_info(),
            ai_dex.to_account_info(),
            token_program.to_account_info(),
        ],
        &[&ai_dex.seeds()],
    )?;
    Ok(())
}

/// Mints a position trade batch token and then removes the mint authority.
///
/// # Arguments
///
/// * `position_trade_batch` - The account representing the position trade batch.
/// * `position_trade_batch_mint` - The mint of the position trade batch token.
/// * `position_trade_batch_token_account` - The token account for the position trade batch.
/// * `token_program` - The token program.
/// * `position_trade_batch_seeds` - The seeds for the position trade batch.
///
/// # Errors
///
/// Returns an error if the mint operation or the removal of the mint authority fails.
pub fn mint_position_trade_batch_token_and_remove_authority<'info>(
    position_trade_batch: &Account<'info, PositionTradeBatch>,
    position_trade_batch_mint: &Account<'info, Mint>,
    position_trade_batch_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    position_trade_batch_seeds: &[&[u8]],
) -> Result<()> {
    mint_position_trade_batch_token(
        position_trade_batch,
        position_trade_batch_mint,
        position_trade_batch_token_account,
        token_program,
        position_trade_batch_seeds,
    )?;
    remove_position_trade_batch_token_mint_authority(
        position_trade_batch,
        position_trade_batch_mint,
        token_program,
        position_trade_batch_seeds,
    )
}

/// Mints a position trade batch token with metadata and removes the mint authority.
///
/// # Arguments
///
/// * `funder` - The account funding the transaction.
/// * `position_trade_batch` - The position trade batch account.
/// * `position_trade_batch_mint` - The mint of the position trade batch token.
/// * `position_trade_batch_token_account` - The position trade batch token account.
/// * `position_trade_batch_metadata` - The metadata account for the position trade batch token.
/// * `metadata_update_auth` - The account authorized to update the metadata.
/// * `metadata_program` - The metadata program.
/// * `token_program` - The token program.
/// * `system_program` - The system program.
/// * `rent` - The rent sysvar.
/// * `position_trade_batch_seeds` - The seeds for the position trade batch.
///
/// # Errors
///
/// Returns an error if the mint, metadata creation, or authority removal fails.
pub fn mint_position_trade_batch_token_with_metadata_and_remove_authority<'info>(
    funder: &Signer<'info>,
    position_trade_batch: &Account<'info, PositionTradeBatch>,
    position_trade_batch_mint: &Account<'info, Mint>,
    position_trade_batch_token_account: &Account<'info, TokenAccount>,
    position_trade_batch_metadata: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    metadata_program: &Program<'info, metadata::Metadata>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
    position_trade_batch_seeds: &[&[u8]],
) -> Result<()> {
    mint_position_trade_batch_token(
        position_trade_batch,
        position_trade_batch_mint,
        position_trade_batch_token_account,
        token_program,
        position_trade_batch_seeds,
    )?;

    let mint_address = position_trade_batch_mint.key().to_string();
    let nft_name = format!(
        "{} {}...{}",
        ADB_METADATA_SYMBOL,
        &mint_address[0..4],
        &mint_address[mint_address.len() - 4..]
    );

    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: position_trade_batch_metadata.to_account_info(),
                mint: position_trade_batch_mint.to_account_info(),
                mint_authority: position_trade_batch.to_account_info(),
                update_authority: metadata_update_auth.to_account_info(),
                payer: funder.to_account_info(),
                rent: rent.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[position_trade_batch_seeds],
        ),
        DataV2 {
            name: nft_name,
            symbol: ADB_METADATA_SYMBOL.to_string(),
            uri: ADB_METADATA_URI.to_string(),
            creators: None,
            seller_fee_basis_points: 0,
            collection: None,
            uses: None
        },
        true,
        false,
        None
    )?;

    remove_position_trade_batch_token_mint_authority(
        position_trade_batch,
        position_trade_batch_mint,
        token_program,
        position_trade_batch_seeds,
    )
}

/// Mints a position trade batch token.
///
/// # Arguments
///
/// * `position_trade_batch` - The account representing the position trade batch.
/// * `position_trade_batch_mint` - The mint of the position trade batch token.
/// * `position_trade_batch_token_account` - The token account for the position trade batch.
/// * `token_program` - The token program.
/// * `position_trade_batch_seeds` - The seeds for the position trade batch.
///
/// # Errors
///
/// Returns an error if the mint operation fails.
fn mint_position_trade_batch_token<'info>(
    position_trade_batch: &Account<'info, PositionTradeBatch>,
    position_trade_batch_mint: &Account<'info, Mint>,
    position_trade_batch_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    position_trade_batch_seeds: &[&[u8]],
) -> Result<()> {
    invoke_signed(
        &mint_to(
            token_program.key,
            position_trade_batch_mint.to_account_info().key,
            position_trade_batch_token_account.to_account_info().key,
            position_trade_batch.to_account_info().key,
            &[],
            1,
        )?,
        &[
            position_trade_batch_mint.to_account_info(),
            position_trade_batch_token_account.to_account_info(),
            position_trade_batch.to_account_info(),
            token_program.to_account_info(),
        ],
        &[position_trade_batch_seeds],
    )?;

    Ok(())
}

/// Removes the mint authority from the position trade batch token.
///
/// # Arguments
///
/// * `position_trade_batch` - The PositionTradeBatch account.
/// * `position_trade_batch_mint` - The mint of the position trade batch token.
/// * `token_program` - The token program.
/// * `position_trade_batch_seeds` - The seeds for the position trade batch.
///
/// # Errors
///
/// Returns an error if the authority removal fails.
fn remove_position_trade_batch_token_mint_authority<'info>(
    position_trade_batch: &Account<'info, PositionTradeBatch>,
    position_trade_batch_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    position_trade_batch_seeds: &[&[u8]],
) -> Result<()> {
    // Invoke the set_authority instruction to remove the mint authority
    invoke_signed(
        &set_authority(
            token_program.key,
            position_trade_batch_mint.to_account_info().key,
            Option::None,
            AuthorityType::MintTokens,
            position_trade_batch.to_account_info().key,
            &[],
        )?,
        &[
            position_trade_batch_mint.to_account_info(),
            position_trade_batch.to_account_info(),
            token_program.to_account_info(),
        ],
        &[position_trade_batch_seeds],
    )?;

    Ok(())
}

/// Burns a single token from the position trade batch token account and closes the account.
///
/// # Arguments
///
/// * `position_trade_batch_authority` - The signer authority for the position trade batch.
/// * `receiver` - The account to receive the remaining funds.
/// * `position_trade_batch_mint` - The mint of the position trade batch token.
/// * `position_trade_batch_token_account` - The position trade batch token account.
/// * `token_program` - The token program.
///
/// # Errors
///
/// Returns an error if the burn or close account operations fail.
pub fn burn_and_close_position_trade_batch_token<'info>(
    position_trade_batch_authority: &Signer<'info>,
    receiver: &UncheckedAccount<'info>,
    position_trade_batch_mint: &Account<'info, Mint>,
    position_trade_batch_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    // use same logic
    burn_and_close_user_position_token(
        position_trade_batch_authority,
        receiver,
        position_trade_batch_mint,
        position_trade_batch_token_account,
        token_program,
    )
}
