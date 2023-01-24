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
1. Explain how Session code is executed in the account's context once a .wasm is installed through the put_deploy \
entry_point.