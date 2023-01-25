# Access and manage access to funds in a Casper Smart Contract
See [session_code](https://github.com/jonas089/C3PRL0CK/tree/master/session_code) to learn how to lock funds up in \
a Smart Contract \
Approved redemption of Tokens from Contract purse
## Entry Points
0. **Call** \
Create a contract purse and transfer [amount] tokens into it.
1. **Approve** \
Approve an account hash => owner of that account can redeem tokens from the contract.
2. **Deposit** \
Transfer tokens from account's main purse to an existing contract purse.
3. **Redeem** \
Redeem tokens form an existing contract purse. Only possible if either in contract purse owner's approval list or the contract purse owner themself.

# Outline Session-code and Contract
Session code is executed in the caller account's context. \
Session code can be used to fund a smart contract / emit a tranfer from \
an account's main purse. \
Smart Contracts have entry points defined that are executed in contract context. \
You can't access an account's main purse from within an Entry Point. \
Use session code ( e.g. the "call" function ) instead. \
To run session code you need to send a .wasm deploy with a "call" function to the get_deploy rpc endpoint.

# Proposed structure for Session code / Smart contract documentation
## From old documentation
1. Writing a basic smart contract in Rust
2. Testing Smart contracts

## New documentation
Goals: \
1. Teach runtime-context so developers understand that session-code is ran in the account-context, where the account's main-purse can be accessed / CSPR can be spent and multisig conditions can be set. \
2. Show how re-useable purses can be created in "Vault" contracts that are funded through session-code. The example "Vault" contract has an approve and redeem Entry Point.

Structure:
1. Explain how Session code is executed in the account's context once a .wasm is installed through the put_deploy
entry_point. \
A suitable example is the account::main_purse and transfer_from_purse_to_account.
2. Show how a purse can be created and stored / re-used in a smart contract's context => smart contract has access rights.
3. Combine **1** and **2** and explain that Session code (**1**) can be used to fund the contract purse created in (**2**).
4. Re-use the example on multisig from the old doc. \
This happens in account-context and is therefore session code.
```
#[no_mangle]
pub extern "C" fn call() {
    // Account hash for the account to be associated.
    let deployment_account: AccountHash = runtime::get_named_arg(ASSOCIATED_ACCOUNT);

    // Add the CA key to half the deployment threshold (i.e 1)
    account::add_associated_key(deployment_account, Weight::new(1)).unwrap_or_revert();

    // Deployment threshold <= Key management threshold.
    // Therefore update the key management threshold value.
    account::set_action_threshold(ActionType::KeyManagement, Weight::new(2)).unwrap_or_revert();

    // Set the deployment threshold to 2 enforcing multisig to send deploys.
    account::set_action_threshold(ActionType::Deployment, Weight::new(2)).unwrap_or_revert();
}
```
5. Compiling Session code: Copy from perivous docs
6. Installing wasm/ executing session code: Copy from previous docs

## Smart Contract testing - ask Karol for more info.
An online IDE or any interface would make debugging contracts a lot more fun and writing tests in runtime is quite painful. I personally prefer testing by running my contracts in NCTL and don't see how it's any more convenient to write tests in the crate. \
Example: \
```
    // Verify the value of count is now 1.
    // count being a value in storage
    let incremented_count = builder
        .query(None, count_key, &[])
        .expect("should be stored value.")
        .as_cl_value()
        .expect("should be cl value.")
        .clone()
        .into_t::<i32>()
        .expect("should be i32.");

    assert_eq!(incremented_count, 1);
```
Writing such a test function is not much better than installing the contract and querying the value using command line. There may however be more complex cases where writing tests in the crate as per the doc can be beneficial. The old documentation is sufficient for explaining tests in crate.


5. Vault contract example 
This contract is two-sided.
1. Smart Contract: \
    A Smart Contract is installed and a new purse is created. \ 
    The purse is stored in the contract's named_keys with access rights set for the contract. \
    On installation an amount is specified that will be transferred to the contract purse as an initial balance. \
    This inital funding transfer from the account_purse happens in the call() function. \
    Endpoints: \
    redeem: withdraw Casper from the contract purse \
    approve: approve an account_hash to withdraw using the redeem entry_point.
2. Session Code: \
    A Session Code that deposits into a Vault-contract's purse. \
    As we want to emit a transfer from the account's main purse to the contract purse, we need to use session-code. \
    Again, Session Code is executed in the account context.
6. Multisig example => from old doc

# Draft




Session code and smart contract code may have similar syntax, but they operate in different contexts. Session code is executed within the context of the caller's account, while smart contract code is installed on the blockchain and executed within the context of the contract. This means that session code only has one entry point, known as the "call" function, while smart contracts can have multiple entry points. \

It's also important to note that when using the storage system in session code, data is stored and retrieved from the caller's account's named keys. On the other hand, in a smart contract context, the storage system reads and writes data from and to the contract's named keys.
**When should you use Session Code?**
1. When transferring funds from the account's main purse
2. When configuring multisig thresholds or assigning weights to keys
3. When you need to call a Smart Contract Entry Point in the account's context
## Example 1: Session Code transfer

```
pub extern "C" fn call():
    let account_purse: URef = account::get_main_purse();
    let amount: U512 = runtime::get_named_arg("amount");
    let recipient_account_hash: AccountHash = runtime::get_named_arg("recipient");
    system::transfer_from_purse_to_account(account_purse, recipient_account_hash, amount, None);

```

This Session code emits a transfer from the account that was used to sign the session deploy to an account_hash that is specified as a runtime argument. \
Other transfer functions in system include:
1. transfer_from_purse_to_purse
2. transfer_to_account
3. transfer_from_purse_to_public

## Compiling Session Code
=> copy from old documentation 
## Use put_deploy to run Session Code
=> copy from old documentation
## Optional: Testing Session Code - This needs to be discussed further with Karol.

## Writing a basic Smart Contract
=> copy counter example from old documentation
## Testing a basic Smart Contract 
=> copy counter example from old documentation

## Writing a Vault Smart Contract
Context Stack overview: \
1. A Contract (C1) is installed
2. A Contract (C1) is called to install a new Contract (C2). (C2= a Vault Contract with a purse under it)
3. Session code is used to transfer funds to the Vault Contract's (C2) purse

Contract (C1) [source](https://github.com/jonas089/C3PRL0CK) \
Install Contract (C1) as per [install smart contracts](FUTURE_LINK_GOES_HERE) and supply an amount as a session arg for funding a Vault Contract (C2) on installation / migration.

### How Contract (C1) works:
Contract (C1) [main.rs](https://github.com/jonas089/C3PRL0CK/blob/master/contract/src/main.rs) \
Contract (C1) holds an Entry Point named "migrate": 
```
#[no_mangle]
pub extern "C" fn migrate(){
    let owner_account: AccountHash = runtime::get_named_arg("owner_account");
    // create a new purse to later be stored in the contract's named keys
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
        // store the newly created purse in the contract's named keys
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
```
We can split this Entry Point up to make it easier to understand. \
### First, it creates a new purse named "destination" in the Contract's (C1) context:
```
    // create a new purse to later be stored in the contract's named keys
    let destination: URef = system::create_purse();
```
### Then adds the newly created "destination" purse to a new Vault Contract's (C2) named keys:
```
    let named_keys = {
        ...
        // store the newly created purse in the contract's named keys
        named_keys.insert(ARG_DESTINATION.to_string(), destination.into());

        named_keys
    };
```
### Lastly the new Contract (C2) is installed on-chain and the "destination" purse is returned from the Contract's (C2) named keys.
```
    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some("child_contract_hash".to_string()),
        Some("child_contract_uref".to_string()),
    );
    let _destination = CLValue::from_t(destination).unwrap_or_revert();
    // return new purse
    runtime::ret(_destination);
```
You will find the new Contract (C2) in the current execution context's named keys. As we are calling a Smart Contract's (C1) "migrate" Entry Point, the context of execution will be the Contract (C1) that holds the "migrate" Entry Point. Therefore we need to query the named keys of the Smart Contract (C1) that holds this Entry Point to find our newly installed Contract (C2) with the purse stored under its named keys. 
1. Query Contract (C1) to find the contract hash of Vault Contract (C2)
2. Query Vault Contract (C2) to find the Vault Contract's purse in named keys
Summary: C1 "migrate" Entry Point is called to install C2 (with purse in named keys) \
C2 returns the purse for use in Session Code.
### Deposit Casper in a Vault Contract through Session Code
To transfer Casper from an account to the "destination" purse, we need a Session Code (S) that is executed in the account's context. We supply the contract_hash of the "Vault" Contract (C2) as a session argument when running the Session Code (S) as follows:
```
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
```
The get_purse Entry Point of Vault Contract (C2) returns the stored purse from a Vault Contract's (C2) named keys.
The contract_hash can be found in the newly installed contract's named keys.
### Redeem from Vault Contract or approve accounts to redeem from Vault Contract
1. Redeem Entry Point:
```
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
```
2. Approve Entry Point:
```
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
```
### Multi Sig Session Code Example
=> copy from old documentation