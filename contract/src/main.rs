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
const APPROVED_ACCOUNTS: &str = "approved";
const OWNER_ACCOUNT: &str = "owner";
#[no_mangle]
pub extern "C" fn migrate_and_fund(){
    // access to this contract should be restricted.
    // this contract should only be called once when installing.
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let destination_name: String = runtime::get_named_arg(ARG_DESTINATION);
    let source: URef = account::get_main_purse();
    let owner_account: AccountHash = runtime::get_caller();
    let approval_list: URef = storage::new_dictionary(APPROVED_ACCOUNTS).unwrap_or_revert();

    // new purse is created
    let destination = system::create_purse();
    system::transfer_from_purse_to_purse(source, destination, amount, None).unwrap_or_revert();

    // store data on chain
    runtime::put_key(&String::from(APPROVED_ACCOUNTS), approval_list.into());
    runtime::put_key(&destination_name, destination.into());
    runtime::put_key(OWNER_ACCOUNT, owner_account.into());    
}

#[no_mangle]
pub extern "C" fn approve(){
    let owner_account_uref: URef = match runtime::get_key(OWNER_ACCOUNT){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let owner_account: AccountHash = storage::read_or_revert(owner_account_uref);
    let new_account: AccountHash = runtime::get_named_arg(ARG_ACCOUNT);
    let approval_list_uref: URef = match runtime::get_key(APPROVED_ACCOUNTS){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let approval_list = storage::dictionary_get::<Vec<AccountHash>>(approval_list_uref, &owner_account.to_string()).unwrap_or_revert();
    let res = match approval_list{
        Some(mut v) => {
            v.push(new_account);
            v
        },
        None => {
            let mut _approval_list: Vec<AccountHash> = Vec::new();
            _approval_list.push(new_account);
            _approval_list
        }
    };
    storage::dictionary_put(approval_list_uref, &owner_account.to_string(), res);
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
    let approval_list_uref: URef = match runtime::get_key(APPROVED_ACCOUNTS){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    let approval_list_option = storage::dictionary_get::<Vec<AccountHash>>(approval_list_uref, &owner_account.to_string()).unwrap_or_revert();
    let approval_list:Vec<AccountHash> = match approval_list_option{
        Some(list) => list,
        None => runtime::revert(ApiError::MissingKey)
    };

    if owner_account != caller && !approval_list.contains(&caller){
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
            vec![Parameter::new("account", CLType::Any)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let redeem = EntryPoint::new(
            "redeem",
            vec![Parameter::new("amount", CLType::U512)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        let deposit = EntryPoint::new(
            "deposit",
            vec![Parameter::new("amount", CLType::U512), Parameter::new("destination", CLType::String)],
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
        Some("contract_hash".to_string()),
        Some("contract_uref".to_string()),
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
