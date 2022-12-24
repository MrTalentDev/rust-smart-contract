#![no_std]
#![no_main]
extern crate alloc;
use alloc::string::String;
use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert
};
use casper_types::{URef, U512, Key, ApiError};
const ARG_DESTINATION: &str = "destination";
#[no_mangle]
pub extern "C" fn redeem(){
    let destination_purse_uref: URef = account::get_main_purse();
    let stored_purse_key: Key = match runtime::get_key(ARG_DESTINATION){
        Some(key) => key,
        None => runtime::revert(ApiError::MissingKey)
    };
    let stored_purse_uref = stored_purse_key.into_uref().unwrap_or_revert();
    //system::transfer_from_purse_to_purse(stored_purse_uref, destination_purse_uref, amount, None);
}
#[no_mangle]
pub extern "C" fn call(){
    //let amount:U512 = runtime::get_named_arg("amount");
    let amount:U512 = U512::from(1000000000 as u64);
    let destination_name: String = runtime::get_named_arg(ARG_DESTINATION);
    let source: URef = account::get_main_purse();
    let destination = system::create_purse();
    system::transfer_from_purse_to_purse(source, destination, amount, None).unwrap_or_revert();
    runtime::put_key(&destination_name, destination.into());
}
