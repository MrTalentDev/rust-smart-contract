#![no_std]
#![no_main]
extern crate alloc;
use alloc::string::String;
use casper_contract::{
    contract_api::{account, runtime, system, storage},
    unwrap_or_revert::UnwrapOrRevert
};
use casper_types::{URef, U512, Key, ApiError, account::{AccountHash, Account}};
const ARG_DESTINATION: &str = "destination";
const ARG_AMOUNT: &str = "amount";
const OWNER_ACCOUNT: &str = "owner";

#[no_mangle]
pub extern "C" fn redeem(){
    let owner_account_uref: URef = match runtime::get_key(OWNER_ACCOUNT){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    }.into_uref().unwrap_or_revert();
    if (storage::read_or_revert(owner_account_uref) != runtime::get_caller()){
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
    if (storage::read_or_revert(owner_account_uref) != runtime::get_caller()){
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
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let destination_name: String = runtime::get_named_arg(ARG_DESTINATION);
    let source: URef = account::get_main_purse();
    let owner_account: AccountHash = runtime::get_caller();
    // new purse is created
    let destination = system::create_purse();
    // amount is transferred into purse
    system::transfer_from_purse_to_purse(source, destination, amount, None).unwrap_or_revert();
    // purse is accessible as a UREF
    runtime::put_key(&destination_name, destination.into());
    runtime::put_key(OWNER_ACCOUNT, owner_account.into());
}
