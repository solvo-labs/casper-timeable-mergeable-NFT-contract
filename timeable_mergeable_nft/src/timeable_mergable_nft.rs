use core::ops::Add;

use alloc::{ string::{ String, ToString }, vec::Vec, vec, boxed::Box, format };

use crate::{ error::Error, utils };

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
    RuntimeArgs,
    runtime_args,
    Parameter,
    U256,
    URef,
};

use casper_types_derive::{ CLTyped, FromBytes, ToBytes };

use casper_contract::contract_api::{ runtime, storage };

use serde::{ Deserialize, Serialize };

// variables
const COLLECTION: &str = "collection";
const METADATA: &str = "metadata";
const TOKEN_ID: &str = "token_id";
const TOKEN_IDS: &str = "token_ids";
const NAME: &str = "name";
const OWNER: &str = "owner";
const NFT_INDEX: &str = "nft_index";

//entry points
const ENTRY_POINT_MINT: &str = "mint";
const ENTRY_POINT_MINT_TIMEABLE_NFT: &str = "mint_timeable_nft";
const ENTRY_POINT_BURN: &str = "burn";
const ENTRY_POINT_MERGE: &str = "merge";
const ENTRY_POINT_INIT: &str = "init";
const ENTRY_POINT_BURN_TIMEABLE_NFT: &str = "burn_timeable_nft";

//dicts
const TIMEABLE_NFTS: &str = "timeable_nfts";

struct MetadataBase {
    name: String,
    description: String,
    asset: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetadataExtended {
    name: String,
    description: String,
    asset: String,
    timeable: bool,
    mergable: bool,
    timestamp: u64,
}

impl ToString for MetadataBase {
    fn to_string(&self) -> String {
        format!(
            r#"{{"name":"{}","description":"{}","asset":"{}"}}"#,
            self.name,
            self.description,
            self.asset
        )
    }
}

#[derive(Clone, Debug, CLTyped, ToBytes, FromBytes)]
struct TimeableNft {
    nft_index: u64,
    timestamp: u64,
    contract_hash: ContractHash,
    burned: bool,
}

#[no_mangle]
pub extern "C" fn merge() {
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let token_ids: Vec<u64> = runtime::get_named_arg(TOKEN_IDS);
    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();
    let caller: AccountHash = runtime::get_caller();

    for &token_id in &token_ids {
        let metadata = get_nft_metadata(collection_hash, token_id);

        let deserialised: MetadataExtended = serde_json_wasm
            ::from_str::<MetadataExtended>(&metadata)
            .unwrap();

        if deserialised.mergable == false {
            runtime::revert(Error::NotMergableNft);
        }

        burn_nft(collection_hash, token_id);
    }

    let merged_token_ids: String = token_ids
        .iter()
        .map(|&token_id| token_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let message = format!("We merged {} token ids", merged_token_ids);

    let merged_nft = MetadataBase {
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
    let caller: AccountHash = runtime::get_caller();

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    let owner = owner_of(collection_hash, token_id);

    if owner != Key::Account(caller) {
        runtime::revert(Error::InvalidOwner);
    }

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
pub extern "C" fn mint_timeable_nft() {
    let metadata: String = runtime::get_named_arg(METADATA);
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let caller: AccountHash = runtime::get_caller();

    let metadata_extended: MetadataExtended = serde_json_wasm
        ::from_str::<MetadataExtended>(&metadata)
        .unwrap();

    if metadata_extended.timeable == false {
        runtime::revert(Error::NotTimeableNft);
    }

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();
    let contract_address = utils::get_current_address();

    let (_, _, new_nft_index): (String, Key, String) = mint_nft_extend(
        collection_hash,
        Key::Account(caller),
        contract_address.into(),
        metadata
    );

    let nfts: URef = *runtime::get_key(TIMEABLE_NFTS).unwrap().as_uref().unwrap();

    let nft_index: U256 = utils::read_from(NFT_INDEX);

    storage::dictionary_put(nfts, &nft_index.to_string(), TimeableNft {
        nft_index: new_nft_index.parse::<u64>().unwrap(),
        timestamp: metadata_extended.timestamp,
        contract_hash: collection_hash,
        burned: false,
    });

    runtime::put_key(NFT_INDEX, storage::new_uref(nft_index.add(1)).into());
}

#[no_mangle]
pub extern "C" fn burn_timeable_nft() {
    let nfts = *runtime::get_key(TIMEABLE_NFTS).unwrap().as_uref().unwrap();
    let nft_index: u64 = utils::read_from(NFT_INDEX);
    let now: u64 = runtime::get_blocktime().into();

    for i in 0..=nft_index {
        let nft = storage::dictionary_get::<TimeableNft>(nfts, &i.to_string()).unwrap().unwrap();

        // if nft.burned == false && now > nft.timestamp {
        burn_nft(nft.contract_hash, nft.nft_index);

        storage::dictionary_put(nfts, &i.to_string(), TimeableNft {
            nft_index: nft.nft_index,
            timestamp: nft.timestamp,
            contract_hash: nft.contract_hash,
            burned: true,
        });
        // }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    storage::new_dictionary(TIMEABLE_NFTS).unwrap_or_default();

    let nft_count = U256::zero();
    runtime::put_key(NFT_INDEX, storage::new_uref(nft_count).into());
}

#[no_mangle]
pub extern "C" fn call() {
    let name = "Dappend CEP-78 Custom NFT Contract";
    let owner: AccountHash = runtime::get_caller();

    let mut named_keys = NamedKeys::new();

    named_keys.insert(NAME.to_string(), storage::new_uref(name.clone()).into());
    named_keys.insert(OWNER.to_string(), storage::new_uref(owner.clone()).into());

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

    let init_entry_point = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let mint_timeable_nft_entry_point = EntryPoint::new(
        ENTRY_POINT_MINT_TIMEABLE_NFT,
        vec![Parameter::new(COLLECTION, CLType::Key), Parameter::new(METADATA, CLType::String)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let burn_timeable_nft_entry_point = EntryPoint::new(
        ENTRY_POINT_BURN_TIMEABLE_NFT,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    entry_points.add_entry_point(mint_entry_point);
    entry_points.add_entry_point(burn_entry_point);
    entry_points.add_entry_point(merge_entry_point);
    entry_points.add_entry_point(init_entry_point);
    entry_points.add_entry_point(mint_timeable_nft_entry_point);
    entry_points.add_entry_point(burn_timeable_nft_entry_point);

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

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INIT, runtime_args! {});
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

pub fn mint_nft_extend(
    contract_hash: ContractHash,
    owner: Key,
    contract_address: Key,
    metadata: String
) -> (String, Key, String) {
    runtime::call_contract::<(String, URef)>(
        contract_hash,
        "register_owner",
        runtime_args! {
            "token_owner" => contract_address,
        }
    );

    runtime::call_contract::<(String, Key, String)>(
        contract_hash,
        "mint",
        runtime_args! {
          "token_owner" => owner,
          "token_meta_data" => metadata
      }
    )
}

pub fn get_nft_metadata(contract_hash: ContractHash, token_id: u64) -> String {
    runtime::call_contract::<String>(
        contract_hash,
        "metadata",
        runtime_args! {
          "token_id" => token_id,
      }
    )
}

pub fn owner_of(contract_hash: ContractHash, token_id: u64) -> Key {
    runtime::call_contract::<Key>(
        contract_hash,
        "owner_of",
        runtime_args! {
            "token_id" => token_id,
        }
    )
}
