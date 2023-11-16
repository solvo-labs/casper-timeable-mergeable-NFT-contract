use core::ops::Add;

use alloc::{ string::{ String, ToString }, vec };

use crate::{ error::Error, utils::{ get_current_address, get_key, self }, enums::Address };

use casper_types::{
    account::AccountHash,
    EntryPoint,
    Key,
    ContractHash,
    EntryPointAccess,
    CLType,
    EntryPointType,
    EntryPoints,
    contracts::NamedKeys,
    U512,
    RuntimeArgs,
    runtime_args,
    URef,
    CLValue,
    Parameter,
};

use casper_contract::contract_api::{ runtime, storage, system };

// variables
const OWNER: &str = "owner";
const COLLECTION: &str = "collection";
const METADATA: &str = "metadata";
const TOKEN_ID: &str = "token_id";

//entry points
const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_BURN: &str = "burn";

#[no_mangle]
pub extern "C" fn burn() {
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let token_id: u64 = runtime::get_named_arg(TOKEN_ID);

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();
    burn_nft(collection_hash, token_id);
}

#[no_mangle]
pub extern "C" fn mint() {
    let metadata: String = runtime::get_named_arg(METADATA);
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let token_owner: Key = runtime::get_named_arg(OWNER);

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();
    mint_nft(collection_hash, token_owner, metadata);
}

#[no_mangle]
pub extern "C" fn call() {
    let named_keys = NamedKeys::new();

    let mut entry_points = EntryPoints::new();

    let mint_entry_point = EntryPoint::new(
        ENTRY_POINT_MINT,
        vec![
            Parameter::new(COLLECTION, CLType::Key),
            Parameter::new(METADATA, CLType::String),
            Parameter::new(OWNER, CLType::Key)
        ],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let burn_entry_point = EntryPoint::new(
        ENTRY_POINT_BURN,
        vec![Parameter::new(COLLECTION, CLType::Key), Parameter::new(TOKEN_ID, CLType::U64)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    entry_points.add_entry_point(mint_entry_point);
    entry_points.add_entry_point(burn_entry_point);

    let name = "dummy";
    let str1 = &name.to_string();

    let str2 = String::from("dappend_nft_package_hash_");
    let str3 = String::from("dappend_nft_access_uref_");
    let str4 = String::from("dappend_nft_contract_hash_");
    let hash_name = str2 + &str1;
    let uref_name = str3 + &str1;
    let contract_hash_text = str4 + &str1;

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_name.to_string()),
        Some(uref_name.to_string())
    );

    runtime::put_key(&contract_hash_text.to_string(), contract_hash.into());
}

pub fn burn_nft(contract_hash: ContractHash, token_id: u64) -> (String, Key, String) {
    runtime::call_contract(
        contract_hash,
        "burn",
        runtime_args! {
        "token_id" => token_id,
    }
    )
}

pub fn mint_nft(contract_hash: ContractHash, owner: Key, metadata: String) -> () {
    runtime::call_contract::<()>(
        contract_hash,
        "mint",
        runtime_args! {
          "token_owner" => owner,
          "token_meta_data" => metadata,
      }
    )
}