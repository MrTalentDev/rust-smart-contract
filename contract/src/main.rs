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
use casper_types::{ContractHash, CLValue, AccessRights, CLType, EntryPointAccess, EntryPointType, URef, U512, Key, ApiError, account::{AccountHash, Account}, contracts::NamedKeys, EntryPoints, EntryPoint, Parameter, runtime_args, RuntimeArgs};
const ARG_DESTINATION: &str = "destination";
const ARG_AMOUNT: &str = "amount";
const ARG_ACCOUNT: &str = "account";
const APPROVED_LIST: &str = "approved_list";
const OWNER_ACCOUNT: &str = "owner";

/* 
    If you want to install multiple child-contracts throught he parent-contract
    replace child_contract_hash and child_contract_uref
    with a unique var, as otherwise a new vault contract will override the existing one.


    migrate() installs a child_contract which is stored in the parent_contract's named_keys.
*/

/*

1. deploy parent contract => installs child contract
2. call entry points of child contract -> query parent contract to find child contract hash
3. experiment and maybe allow for multiple child contracts to be installed

*/

// in parent context
#[no_mangle]
// install a new vault as a "child_contract" under the "parent_contract's" named_keys
// advantage: keys don't occupy space in account's named_keys
pub extern "C" fn migrate(){
    let owner_account: AccountHash = runtime::get_named_arg("owner_account");
    let destination: URef = system::create_purse();
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
        let get_purse = EntryPoint::new(
            "get_purse",
            vec![],
            CLType::URef,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        entry_points.add_entry_point(approve);
        entry_points.add_entry_point(redeem);
        entry_points.add_entry_point(get_purse);
        entry_points
    };
    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(OWNER_ACCOUNT.to_string(), owner_account.into());
        let approved_list = storage::new_dictionary(APPROVED_LIST).unwrap_or_revert();
        named_keys.insert(APPROVED_LIST.to_string(), approved_list.into());
        named_keys.insert(ARG_DESTINATION.to_string(), destination.into());

        named_keys
    };
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("child_contract_hash".to_string()),
        Some("child_contract_uref".to_string()),
    );
    let _destination = CLValue::from_t(destination).unwrap_or_revert();
    // return new purse for this child contract
    runtime::ret(_destination);
}

#[no_mangle]
pub extern "C" fn get_purse(){
    let purse_uref: URef = match runtime::get_key(ARG_DESTINATION){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    };
    let purse: URef = storage::read_or_revert(purse_uref);
    let _purse = CLValue::from_t(purse).unwrap_or_revert();
    runtime::ret(_purse);
}

// in parent or child context
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
    let stored_purse_uref: URef = match runtime::get_key(ARG_DESTINATION){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    system::transfer_from_purse_to_account(stored_purse_uref, caller, amount, None);
}

// in account context
#[no_mangle]
pub extern "C" fn call(){
    let entry_points = {
        // approve, redeem and get_purse exist in the parent_contract's named_keys, but only so that they can be "found"
        // when installing the child_contract. 
        // only the migrate entry_point is supposed to be called in the parent's context.
        let mut entry_points = EntryPoints::new();
        // passed to child_contract
        let approve = EntryPoint::new(
            "approve",
            vec![Parameter::new(ARG_ACCOUNT, CLType::Any)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        // passed to child_contract
        let redeem = EntryPoint::new(
            "redeem",
            vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        // passed to child_contract
        let get_purse = EntryPoint::new(
            "get_purse",
            vec![],
            CLType::URef,
            EntryPointAccess::Public,
            EntryPointType::Contract
        );
        // call this entry point on the parent_contract to install a new instance of the child_contract.
        // the child_contract is the "vault".
        // the parent contract holds N child_contracts in it's named_keys.
        let migrate = EntryPoint::new(
            "migrate",
            vec![Parameter::new("owner_account", CLType::Key), Parameter::new("destination", CLType::URef)],
            CLType::URef,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        entry_points.add_entry_point(approve);
        entry_points.add_entry_point(redeem);
        entry_points.add_entry_point(get_purse);
        entry_points.add_entry_point(migrate);
        entry_points
    };
    let named_keys = NamedKeys::new();
    
    // install parent under account's named_keys
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("parent_contract_hash".to_string()),
        Some("parent_contract_uref".to_string()),
    );

    let source: URef = account::get_main_purse();
    let owner_account: AccountHash = runtime::get_caller();
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);

    let contract_purse:URef = runtime::call_contract::<URef>(
        contract_hash,
        "migrate",
        runtime_args! {
            "owner_account" => owner_account
        },
    );
    // initial funding of contract purse by contract installer.
    system::transfer_from_purse_to_purse(source, contract_purse, amount, None).unwrap_or_revert();
}