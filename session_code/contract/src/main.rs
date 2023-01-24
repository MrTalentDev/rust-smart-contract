#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::string::String;

use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, Key, ContractHash, U512, URef};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash: ContractHash = runtime::get_named_arg("contract_hash");
    let amount: U512 = runtime::get_named_arg("amount");
    let source: URef = account::get_main_purse();
    let contract_purse:URef = runtime::call_contract::<URef>(
        contract_hash,
        "get_purse",
        runtime_args! {
        },
    );
    system::transfer_from_purse_to_purse(source, contract_purse, amount, None);
}
