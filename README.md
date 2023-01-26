# Access and manage access to funds in a Casper Smart Contract
See [session_code](https://github.com/jonas089/C3PRL0CK/tree/master/session_code) to learn how to lock funds up in a Smart Contract
## Entry Points
0. **Call** \
Create a contract purse and transfer [amount] tokens into it.
1. **Approve** \
Approve an account hash => owner of that account can redeem tokens from the contract.
2. **Deposit** \
Transfer tokens from account's main purse to an existing contract purse.
3. **Redeem** \
Redeem tokens form an existing contract purse. Only possible if either in contract purse owner's approval list or the contract purse owner themself.
