use core::ops::Add;

use alloc::{ string::{ String, ToString }, vec::Vec, vec, boxed::Box, format };

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
const COLLECTION: &str = "collection";
const METADATA: &str = "metadata";
const TOKEN_ID: &str = "token_id";
const TOKEN_IDS: &str = "token_ids";
const NAME: &str = "name";

//entry points
const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_BURN: &str = "burn";
const ENTRY_POINT_MERGE: &str = "merge";

struct Metadata {
    name: String,
    description: String,
    asset: String,
}

impl ToString for Metadata {
    fn to_string(&self) -> String {
        format!(
            r#"{{"name":"{}","description":"{}","asset":"{}"}}"#,
            self.name,
            self.description,
            self.asset
        )
    }
}

#[no_mangle]
pub extern "C" fn merge() {
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let token_ids: Vec<u64> = runtime::get_named_arg(TOKEN_IDS);
    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();
    let caller: AccountHash = runtime::get_caller();

    for &token_id in &token_ids {
        burn_nft(collection_hash, token_id);
    }

    let merged_token_ids: String = token_ids
        .iter()
        .map(|&token_id| token_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let message = format!("We merged {} token ids", merged_token_ids);

    let merged_nft = Metadata {
        name: String::from("Dappend Merged Nft"),
        description: message,
        asset: String::from(
            "https://ipfs.io/ipfs/bafkreieo73godjrxffwufu36rks7ro3uno4zrkm5vne4bl6jhojyxvhjci"
        ),
    };

    mint_nft(collection_hash, Key::Account(caller), merged_nft.to_string());
}

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
    let caller: AccountHash = runtime::get_caller();

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();
    mint_nft(collection_hash, Key::Account(caller), metadata);
}

#[no_mangle]
pub extern "C" fn call() {
    let name = "Dappend CEP-78 Custom NFT Contract";

    let mut named_keys = NamedKeys::new();

    named_keys.insert(NAME.to_string(), storage::new_uref(name.clone()).into());

    let mut entry_points = EntryPoints::new();

    let mint_entry_point = EntryPoint::new(
        ENTRY_POINT_MINT,
        vec![Parameter::new(COLLECTION, CLType::Key), Parameter::new(METADATA, CLType::String)],
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

    let merge_entry_point = EntryPoint::new(
        ENTRY_POINT_MERGE,
        vec![
            Parameter::new(COLLECTION, CLType::Key),
            Parameter::new(TOKEN_IDS, CLType::List(Box::new(CLType::U64)))
        ],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    entry_points.add_entry_point(mint_entry_point);
    entry_points.add_entry_point(burn_entry_point);
    entry_points.add_entry_point(merge_entry_point);

    let hash_name = String::from("dappend_nft_package_hash");
    let uref_name = String::from("dappend_nft_access_uref");
    let contract_hash_text = String::from("dappend_nft_contract_hash");

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_name.to_string()),
        Some(uref_name.to_string())
    );

    runtime::put_key(&contract_hash_text.to_string(), contract_hash.into());
}

pub fn burn_nft(contract_hash: ContractHash, token_id: u64) -> () {
    runtime::call_contract::<()>(
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
