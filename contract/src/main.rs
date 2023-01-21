#![no_std]
#![no_main]
extern crate alloc;
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{account, runtime, system, storage},
    unwrap_or_revert::UnwrapOrRevert
};
use casper_types::{CLType, EntryPointAccess, EntryPointType, URef, U512, Key, ApiError, account::{AccountHash, Account}, contracts::NamedKeys, EntryPoints, EntryPoint, Parameter, runtime_args, RuntimeArgs};
const ARG_DESTINATION: &str = "destination";
const ARG_AMOUNT: &str = "amount";
const ARG_ACCOUNT: &str = "account";
const APPROVED_LIST: &str = "approved_list";
const OWNER_ACCOUNT: &str = "owner";

/*

1. deploy parent contract => installs child contract
2. call entry points of child contract -> query parent contract to find child contract hash
3. experiment and maybe allow for multiple child contracts to be installed


*/

#[no_mangle]
pub extern "C" fn migrate_and_fund(){
    // access to this contract should be restricted.
    // this contract should only be called once when installing.
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let source: URef = account::get_main_purse();
    let owner_account: AccountHash = runtime::get_caller();

    // new purse is created
    let destination = system::create_purse();
    system::transfer_from_purse_to_purse(source, destination, amount, None).unwrap_or_revert();

    // store data on chain
    //runtime::put_key(&String::from(APPROVED_LIST), approved_list.into());
    //runtime::put_key(&destination_name, destination.into());
    //runtime::put_key(OWNER_ACCOUNT, owner_account.into());    

    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let approve = EntryPoint::new(
            "approve",
            vec![Parameter::new(ARG_ACCOUNT, CLType::Any)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let redeem = EntryPoint::new(
            "redeem",
            vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let deposit = EntryPoint::new(
            "deposit",
            vec![Parameter::new(ARG_AMOUNT, CLType::U512), Parameter::new(ARG_DESTINATION, CLType::String)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        entry_points.add_entry_point(approve);
        entry_points.add_entry_point(redeem);
        entry_points.add_entry_point(deposit);
        entry_points
    };
    let named_keys = {
        let mut named_keys = NamedKeys::new();
        // store the installer of the child contract
        // question: is this the parent contract or the account_hash of the user calling?
        // for now not relevant as migrate can only be called from call() in parent.
        named_keys.insert(OWNER_ACCOUNT.to_string(), owner_account.into());
        let approved_list = storage::new_dictionary(APPROVED_LIST).unwrap_or_revert();
        named_keys.insert(APPROVED_LIST.to_string(), approved_list.into());
        // Warning: if key exists on different contract, deploy will fail ? to be investigated.
        let destination_uref = storage::new_uref(ARG_DESTINATION);
        named_keys.insert(ARG_DESTINATION.to_string(), destination.into());

        named_keys
    };
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("child_contract_hash".to_string()),
        Some("child_contract_uref".to_string()),
    );
}

#[no_mangle]
pub extern "C" fn approve(){
    let owner_account_uref: URef = match runtime::get_key(OWNER_ACCOUNT){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let owner_account: AccountHash = storage::read_or_revert(owner_account_uref);
    let new_account: AccountHash = runtime::get_named_arg(ARG_ACCOUNT);
    let approved_list_uref: URef = match runtime::get_key(APPROVED_LIST){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let approved_list = storage::dictionary_get::<Vec<AccountHash>>(approved_list_uref, &owner_account.to_string()).unwrap_or_revert();
    let res = match approved_list{
        Some(mut v) => {
            v.push(new_account);
            v
        },
        None => {
            let mut _approved_list: Vec<AccountHash> = Vec::new();
            _approved_list.push(new_account);
            _approved_list
        }
    };
    storage::dictionary_put(approved_list_uref, &owner_account.to_string(), res);
}

#[no_mangle]
pub extern "C" fn redeem(){
    let caller: AccountHash = runtime::get_caller();
    let owner_account_uref: URef = match runtime::get_key(OWNER_ACCOUNT){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let owner_account: AccountHash = storage::read_or_revert(owner_account_uref);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let approved_list_uref: URef = match runtime::get_key(APPROVED_LIST){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let approved_list_option = storage::dictionary_get::<Vec<AccountHash>>(approved_list_uref, &owner_account.to_string()).unwrap_or_revert();
    let approved_list:Vec<AccountHash> = match approved_list_option{
        Some(list) => list,
        None => runtime::revert(ApiError::MissingKey)
    };

    if owner_account != caller && !approved_list.contains(&caller){
        runtime::revert(ApiError::PermissionDenied);
    };
    let destination_purse_uref: URef = account::get_main_purse();
    let stored_purse_uref: URef = match runtime::get_key(ARG_DESTINATION){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    system::transfer_from_purse_to_purse(stored_purse_uref, destination_purse_uref, amount, None);
}
#[no_mangle]
pub extern "C" fn deposit(){
    let owner_account_uref: URef = match runtime::get_key(OWNER_ACCOUNT){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
   let owner_account: AccountHash = storage::read_or_revert(owner_account_uref);
    if (owner_account != runtime::get_caller()){
        runtime::revert(ApiError::PermissionDenied);
    };
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let source: URef = account::get_main_purse();
    let stored_purse_uref: URef = match runtime::get_key(ARG_DESTINATION){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    system::transfer_from_purse_to_purse(source, stored_purse_uref, amount, None);
}
#[no_mangle]
pub extern "C" fn call(){
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let approve = EntryPoint::new(
            "approve",
            vec![Parameter::new(ARG_ACCOUNT, CLType::Any)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let redeem = EntryPoint::new(
            "redeem",
            vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let deposit = EntryPoint::new(
            "deposit",
            vec![Parameter::new(ARG_AMOUNT, CLType::U512), Parameter::new(ARG_DESTINATION, CLType::String)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let migrate = EntryPoint::new(
            "migrate",
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        entry_points.add_entry_point(approve);
        entry_points.add_entry_point(redeem);
        entry_points.add_entry_point(deposit);
        entry_points.add_entry_point(migrate);
        entry_points
    };
    let named_keys = NamedKeys::new();
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("parent_contract_hash".to_string()),
        Some("parent_contract_uref".to_string()),
    );
    // call the contract's migration endpoint ("migrate") to store URefs 
    // within the context of contract, rather than account.
    runtime::call_contract::<()>(
        contract_hash,
        "migrate",
        runtime_args! {
        },
    );
}
