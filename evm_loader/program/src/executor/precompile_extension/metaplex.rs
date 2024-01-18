#![allow(clippy::unnecessary_wraps)]

use std::any::Any;
use std::convert::{Into, TryInto};

use ethnum::U256;
use maybe_async::maybe_async;
use mpl_token_metadata::types::{Creator, DataV2};
use ripemd::digest::typenum::cmp;
use solana_program::{pubkey::Pubkey, rent::Rent, sysvar::Sysvar};

use crate::{
    account::ACCOUNT_SEED_VERSION,
    account_storage::AccountStorage,
    error::{Error, Result},
    executor::ExecutorState,
    types::Address,
};

// "[0xc5, 0x73, 0x50, 0xc6]": "createMetadata(bytes32,string,string,string)"
// "[0x4a, 0xe8, 0xb6, 0x6b]": "createMasterEdition(bytes32,uint64)"
// "[0xf7, 0xb6, 0x37, 0xbb]": "isInitialized(bytes32)"
// "[0x23, 0x5b, 0x2b, 0x94]": "isNFT(bytes32)"
// "[0x9e, 0xd1, 0x9d, 0xdb]": "uri(bytes32)"
// "[0x69, 0x1f, 0x34, 0x31]": "name(bytes32)"
// "[0x6b, 0xaa, 0x03, 0x30]": "symbol(bytes32)"

#[maybe_async]
pub async fn metaplex<B: AccountStorage>(
    state: &mut ExecutorState<'_, B>,
    address: &Address,
    input: &[u8],
    context: &crate::evm::Context,
    is_static: bool,
) -> Result<Vec<u8>> {
    if context.value != 0 {
        return Err(Error::Custom("Metaplex: value != 0".to_string()));
    }

    if &context.contract != address {
        return Err(Error::Custom(
            "Metaplex: callcode or delegatecall is not allowed".to_string(),
        ));
    }

    let (selector, input) = input.split_at(4);
    let selector: [u8; 4] = selector.try_into()?;

    match selector {
        [0xc5, 0x73, 0x50, 0xc6] => {
            // "createMetadata(bytes32,string,string,string)"
            if is_static {
                return Err(Error::StaticModeViolation(*address));
            }

            let mint = read_pubkey(input)?;
            let name = read_string(input, 32, 256)?;
            let symbol = read_string(input, 64, 256)?;
            let uri = read_string(input, 96, 1024)?;

            create_metadata(context, state, mint, name, symbol, uri)
        }
        [0x4a, 0xe8, 0xb6, 0x6b] => {
            // "createMasterEdition(bytes32,uint64)"
            if is_static {
                return Err(Error::StaticModeViolation(*address));
            }

            let mint = read_pubkey(input)?;
            let max_supply = read_u64(&input[32..])?;

            create_master_edition(context, state, mint, Some(max_supply))
        }
        [0xf7, 0xb6, 0x37, 0xbb] => {
            // "isInitialized(bytes32)"
            let mint = read_pubkey(input)?;
            is_initialized(context, state, mint).await
        }
        [0x23, 0x5b, 0x2b, 0x94] => {
            // "isNFT(bytes32)"
            let mint = read_pubkey(input)?;
            is_nft(context, state, mint).await
        }
        [0x9e, 0xd1, 0x9d, 0xdb] => {
            // "uri(bytes32)"
            let mint = read_pubkey(input)?;
            uri(context, state, mint).await
        }
        [0x69, 0x1f, 0x34, 0x31] => {
            // "name(bytes32)"
            let mint = read_pubkey(input)?;
            token_name(context, state, mint).await
        }
        [0x6b, 0xaa, 0x03, 0x30] => {
            // "symbol(bytes32)"
            let mint = read_pubkey(input)?;
            symbol(context, state, mint).await
        }
        _ => Err(Error::UnknownPrecompileMethodSelector(*address, selector)),
    }
}

#[inline]
fn read_u64(input: &[u8]) -> Result<u64> {
    if input.len() < 32 {
        return Err(Error::OutOfBounds);
    }
    U256::from_be_bytes(*arrayref::array_ref![input, 0, 32])
        .try_into()
        .map_err(Into::into)
}

#[inline]
fn read_pubkey(input: &[u8]) -> Result<Pubkey> {
    if input.len() < 32 {
        return Err(Error::OutOfBounds);
    }
    Ok(Pubkey::new_from_array(*arrayref::array_ref![input, 0, 32]))
}

#[inline]
fn read_string(input: &[u8], offset_position: usize, max_length: usize) -> Result<String> {
    if input.len() < offset_position + 32 {
        return Err(Error::OutOfBounds);
    }
    let offset: usize =
        U256::from_be_bytes(*arrayref::array_ref![input, offset_position, 32]).try_into()?;
    if input.len() < offset.saturating_add(32) {
        return Err(Error::OutOfBounds);
    }
    let length = U256::from_be_bytes(*arrayref::array_ref![input, offset, 32]).try_into()?;
    if length > max_length {
        return Err(Error::OutOfBounds);
    }

    let begin = offset.saturating_add(32);
    let end = begin.saturating_add(length);

    if input.len() < end {
        return Err(Error::OutOfBounds);
    }
    let data = input[begin..end].to_vec();
    String::from_utf8(data).map_err(|_| Error::Custom("Invalid utf8 string".to_string()))
}

fn create_metadata<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<B>,
    mint: Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Result<Vec<u8>> {
    let signer = context.caller;
    let (signer_pubkey, bump_seed) = state.backend.contract_pubkey(signer);

    let seeds = vec![
        vec![ACCOUNT_SEED_VERSION],
        signer.as_bytes().to_vec(),
        vec![bump_seed],
    ];

    let (metadata_pubkey, _) = mpl_token_metadata::accounts::Metadata::find_pda(&mint);

    let instruction = mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
        .metadata(metadata_pubkey)
        .mint(mint)
        .mint_authority(signer_pubkey)
        .payer(state.backend.operator())
        .update_authority(signer_pubkey)
        .data(DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(vec![
                Creator {
                    address: *state.backend.program_id(),
                    verified: false,
                    share: 0,
                },
                Creator {
                    address: signer_pubkey,
                    verified: true,
                    share: 100,
                },
            ]),
            collection: None,
            uses: None,
        })
        .is_mutable(false)
        .build();

    let rent = Rent::get()?;
    let fee = rent.minimum_balance(MAX_METADATA_LEN) + CREATE_FEE;

    state.queue_external_instruction(instruction, seeds, fee);

    Ok(metadata_pubkey.to_bytes().to_vec())
}

fn create_master_edition<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<B>,
    mint: Pubkey,
    max_supply: Option<u64>,
) -> Result<Vec<u8>> {
    let signer = context.caller;
    let (signer_pubkey, bump_seed) = state.backend.contract_pubkey(signer);

    let seeds = vec![
        vec![ACCOUNT_SEED_VERSION],
        signer.as_bytes().to_vec(),
        vec![bump_seed],
    ];

    let (metadata_pubkey, _) = mpl_token_metadata::accounts::metadata::Metadata::find_pda(&mint);
    let (edition_pubkey, _) =
        mpl_token_metadata::accounts::master_edition::MasterEdition::find_pda(&mint);

    let mut builder = mpl_token_metadata::instructions::create_master_edition_v3::CreateMasterEditionV3Builder::new()
        .edition(edition_pubkey)
        .mint(mint)
        .update_authority(signer_pubkey)
        .mint_authority(signer_pubkey)
        .metadata(metadata_pubkey)
        .payer(state.backend.operator());

    if let Some(max_supply) = max_supply {
        builder = builder.max_supply(max_supply);
    }

    let instruction = builder.build();

    let rent = Rent::get()?;
    let fee = rent.minimum_balance(MAX_MASTER_EDITION_LEN) + CREATE_FEE;

    state.queue_external_instruction(instruction, seeds, fee);

    Ok(edition_pubkey.to_bytes().to_vec())
}

#[maybe_async]
async fn is_initialized<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<'_, B>,
    mint: Pubkey,
) -> Result<Vec<u8>> {
    let is_initialized = metadata(context, state, mint)
        .await?
        .map_or_else(|| false, |_| true);

    Ok(to_solidity_bool(is_initialized))
}

#[maybe_async]
async fn is_nft<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<'_, B>,
    mint: Pubkey,
) -> Result<Vec<u8>> {
    let is_nft = metadata(context, state, mint).await?.map_or_else(
        || false,
        |m| m.token_standard == Some(TokenStandard::NonFungible),
    );

    Ok(to_solidity_bool(is_nft))
}

#[maybe_async]
async fn uri<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<'_, B>,
    mint: Pubkey,
) -> Result<Vec<u8>> {
    let uri = metadata(context, state, mint)
        .await?
        .map_or_else(String::new, |m| m.data.uri);

    Ok(to_solidity_string(uri.trim_end_matches('\0')))
}

#[maybe_async]
async fn token_name<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<'_, B>,
    mint: Pubkey,
) -> Result<Vec<u8>> {
    let token_name = metadata(context, state, mint)
        .await?
        .map_or_else(String::new, |m| m.data.name);

    Ok(to_solidity_string(token_name.trim_end_matches('\0')))
}

#[maybe_async]
async fn symbol<B: AccountStorage>(
    context: &crate::evm::Context,
    state: &mut ExecutorState<'_, B>,
    mint: Pubkey,
) -> Result<Vec<u8>> {
    let symbol = metadata(context, state, mint)
        .await?
        .map_or_else(String::new, |m| m.data.symbol);

    Ok(to_solidity_string(symbol.trim_end_matches('\0')))
}

#[maybe_async]
async fn metadata<B: AccountStorage>(
    _context: &crate::evm::Context,
    state: &mut ExecutorState<'_, B>,
    mint: Pubkey,
) -> Result<Option<Metadata>> {
    let (metadata_pubkey, _) = mpl_token_metadata::accounts::metadata::Metadata::find_pda(&mint);
    let metadata_account = state.external_account(metadata_pubkey).await?;

    let result = {
        if mpl_token_metadata::check_id(&metadata_account.owner) {
            let metadata = Metadata::safe_deserialize(&metadata_account.data);
            metadata.ok()
        } else {
            None
        }
    };
    Ok(result)
}

fn to_solidity_bool(v: bool) -> Vec<u8> {
    let mut result = vec![0_u8; 32];
    result[31] = u8::from(v);
    result
}

fn to_solidity_string(s: &str) -> Vec<u8> {
    // String encoding
    // 32 bytes - offset
    // 32 bytes - length
    // length + padding bytes - data

    let data_len = if s.len() % 32 == 0 {
        std::cmp::max(s.len(), 32)
    } else {
        ((s.len() / 32) + 1) * 32
    };

    let mut result = vec![0_u8; 32 + 32 + data_len];

    result[31] = 0x20; // offset - 32 bytes

    let length = U256::new(s.len() as u128);
    result[32..64].copy_from_slice(&length.to_be_bytes());

    result[64..64 + s.len()].copy_from_slice(s.as_bytes());

    result
}
