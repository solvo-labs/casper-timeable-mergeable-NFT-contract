#![no_std]
#![no_main]

extern crate alloc;

use casper_contract::{ contract_api::{ runtime, system }, unwrap_or_revert::UnwrapOrRevert };
use casper_types::{ U512, Key, account::AccountHash, runtime_args, ContractHash, RuntimeArgs };
use alloc::string::String;

const NFT_CONTRACT_HASH: &str = "nft_contract_hash";
const COLLECTION: &str = "collection";
const METADATA: &str = "metadata";
const AMOUNT: &str = "amount";
const TARGET_ADDRESS: &str = "target_address";

const ENTRY_POINT_MINT_TIMEABLE_NFT: &str = "mint_timeable_nft";
const ENTRY_POINT_GET_FEE_WALLET: &str = "get_fee_wallet";

#[no_mangle]
pub extern "C" fn call() {
    let nft_contract_hash: ContractHash = runtime::get_named_arg(NFT_CONTRACT_HASH);
    let metadata: String = runtime::get_named_arg(METADATA);
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let target_address: Key = runtime::get_named_arg(TARGET_ADDRESS);

    let amount: U512 = runtime::get_named_arg(AMOUNT);

    let account_key: Key = runtime::call_contract::<Key>(
        nft_contract_hash,
        ENTRY_POINT_GET_FEE_WALLET,
        runtime_args! {}
    );

    let account_hash: AccountHash = account_key.into_account().unwrap_or_revert();

    system::transfer_to_account(account_hash, amount, None).unwrap_or_revert();

    runtime::call_contract::<()>(
        nft_contract_hash,
        ENTRY_POINT_MINT_TIMEABLE_NFT,
        runtime_args! {
            "metadata" => metadata,
            "collection" => collection,
            "target_address" => target_address
        }
    );
}
