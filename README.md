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
You can't access an account's main purse from within an entry point. \
Use session code ( e.g. the "call" function ) instead. \
To run session code you need to send a .wasm deploy with a "call" function to the get_deploy rpc endpoint. \

# Proposed structure for Session code / Smart contract documentation
1. Explain how Session code is executed in the account's context once a .wasm is installed through the put_deploy
entry_point. A suitable example is the account::main_purse and transfer_from_purse_to_account.
2. Show how a purse can be created and stored / re-used in a smart contract's context => smart contract has access rights.
3. Combine **1** and **2** and explain that Session code (**1**) can be used to fund the contract purse created in (**2**).
4. Re-use the example on multisig from the old doc. 
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