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
use casper_types::{CLValue, AccessRights, CLType, EntryPointAccess, EntryPointType, URef, U512, Key, ApiError, account::{AccountHash, Account}, contracts::NamedKeys, EntryPoints, EntryPoint, Parameter, runtime_args, RuntimeArgs};
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

// in parent context
#[no_mangle]
pub extern "C" fn migrate(){
    // TBD: restrict access to this entry_point of parent contract.
    // this may not work.
    let owner_account: AccountHash = runtime::get_named_arg("owner_account");
    // default value for contract purse
    
    // let destination: AccountHash = AccountHash::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    // before this operation, stored_purse_uref is a default value.
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
// Whether or not this is successful depends on the purse 
// access rights. I currently assume that a purse is created
// within a contract's context. Therefore the creating contract
// should have full control over the purse.
// If the redeem entry_point of the child contract 
// executes successfully, the assumption is likely correct.

// More open questions:
// Can the parent contract spend funds that are in the purse?
// Can access rights be configured manually when creating a new purse? - check Casper_Types for this.

// in parent or child context
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

// in parent or child context
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

// in account context
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
            vec![Parameter::new("owner_account", CLType::Key), Parameter::new("destination", CLType::URef)],
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
    let source: URef = account::get_main_purse();
    let owner_account: AccountHash = runtime::get_caller();
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    /*  this needs to be done manually due to runtime context.
    -> a more complex implementation using the caller stack might fix this.
    */
        
    // account::get_main_purse() causes an invalid Context error.

    
    // override default value with newly created purse
    // storage::write(stored_purse_uref, destination);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    // call the new contract.
    // this will create a new purse under the new contract.
    let contract_purse:URef = runtime::call_contract::<URef>(
        contract_hash,
        "migrate",
        runtime_args! {
            "owner_account" => owner_account
        },
    );
    // fund the new contract's purse
    system::transfer_from_purse_to_purse(source, contract_purse, amount, None).unwrap_or_revert();
}

/* 

    tbd: write session code that takes the child contract as input & is used to fund the purse.
    additional entry points for child contract:
    1. get_purse
    ! Still assuming the purse is owned by the child contract.

    session code will look like this:
        get contract purse a
        get main purse b
        transfer_from_purse_to_purse (a, b)


*/