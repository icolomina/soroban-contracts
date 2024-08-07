## Soroban contract examples

Here you can find soroban contract examples. I try to improve and update these contracts continuously with new features and feedback provided by other developers.

### Ballot without token
This contract manages a ballot process following a custodial approach. Allowed-to-vote users are stored in the contract storage. When a user wants to vote, He does not need to sign a transactión with his wallet but the application would be in charge of storing the vote in the contract. 

### Ballot
This contract also manages a ballot process but, in this case, the user must hold a token to be able to vote. The token is defined by the BallotToken contract (ballot/BallotToken). The user must sign the transaction with his wallet since authorization is required and, before storing the vote, the contract ensures the user address holds the token checking the balance. 

### House Purchase
This contract manages a house purchase between buyer and seller. The contract allows the buyer to send the first payment and the buyer (after receiving the first payment) to propose a meeting to formalize the purchase. After formalizing, the contract allows the seller to transfer the rest of the payment.

### Paid Account
The Paid Account contract allows users to deposit tokens (a previously established token) in the contract address and earn daily interest. The contract provides functions for administrators so that the contract always has funds when users request withdrawals.

### Simple deposit
A contract to make a simple deposit to the contract address.

-----------------------------------------------------------------------------------

**IMPORTANT**: These contracts have a test suite but they have not been audited. They can serve as a base for learning but not for being used directly 
in a real application without being audited first.

You can read about these contracts in my dev.to blog:

- **Ballot**: https://dev.to/icolomina/building-a-ballot-contract-using-soroban-plataform-and-rust-sdk-1hg1
- **Ballot with token**: https://dev.to/icolomina/using-tokenization-to-control-a-soroban-voting-smart-contract-3lm6
- **House Purchase**: https://dev.to/icolomina/creating-a-dapp-using-php-and-a-house-purchase-soroban-smart-contract-38f1

> The House Purchase article link shows how to connect to the contract using PHP. It's also a good way to learn how the contract works

- **Paid Account**: Comming soon
- **Simple Deposit**: https://dev.to/icolomina/making-deposits-to-an-smart-contract-using-php-symfony-and-the-soroban-technology-4f10

> The Simple Deposit article link shows an explanation about the contract and how to interact with it using a PHP / Symfony application.

## Test the contracts

To test the contracts, you must prepare first your environment. Follow the [soroban official documentation](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup) to achieve it.
After having the environment ready, follow the next steps:

### Build the contract

```shell
cargo build
```

### Test the contract
```shell
cargo test
```

> The last command must be executed inside the contract root folder. For instance: *soroban-contracts/paid_account*.
