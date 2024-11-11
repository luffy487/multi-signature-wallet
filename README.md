# Multi-Signature Wallet on Solana Blockchain

## Project Description

The **Multi-Signature Wallet on Solana** is a decentralized application (DApp) designed to create, manage, and execute transactions securely through a multi-signature mechanism. This project is built using Anchor, a framework for Solana programs, and leverages the Solana blockchain's security and speed to ensure efficient transaction management among multiple authorized users.

### Key Features:

1. **Create Multi-Signature Wallets**:
   - Users can create multi-signature wallets by specifying the wallet's name, a threshold number of required approvals, and a list of authorized public keys.
   - The wallet is initialized by one of the users, and the rest of the signers must be included during wallet creation.
   - Emits a `WalletCreated` event upon successful creation.

2. **Propose Multi-Signature Transactions**:
   - Wallet owners can propose a transaction by specifying a recipient, amount, and transaction name.
   - Checks to ensure the proposed amount is valid and the proposer is an authorized wallet user.
   - Emits a `TransactionCreated` event when a transaction is proposed.

3. **Sign and Approve Transactions**:
   - Authorized users can sign a proposed transaction, and each signature is recorded.
   - The transaction's progress is tracked, and once the number of signatures meets or exceeds the threshold, the transaction is eligible for execution.
   - Emits a `TransactionSigned` event after each signature.

4. **Execute Transactions**:
   - If the threshold of approvals is met, the transaction is executed, transferring the specified amount from the wallet to the recipient.
   - Ensures recipient validity before execution.
   - Emits a `TransactionExecuted` event when the transaction is completed.

5. **Transfer SOL to Wallet**:
   - Users can transfer SOL to the wallet to fund transactions and ensure the wallet has sufficient balance.

### Smart Contract Structure:

- **Accounts**:
  - `WalletAccount`: Stores wallet information, including its name, creator, list of users, and the approval threshold.
  - `WalletTransaction`: Holds transaction data such as name, amount, recipient, and number of completed signers.
  - `TransactionSignature`: Represents each recorded signature for proposed transactions.
- **Context Structures**:
  - `CreateWallet`, `CreateTransaction`, `SignTheTransaction`, `TransferSolToWallet`: Define the context and authorization checks for each operation.
- **Events**:
  - `WalletCreated`, `TransactionCreated`, `TransactionSigned`, `TransactionExecuted`: Provide logs for tracking wallet creation, transaction proposals, approvals, and execution.
- **Error Handling**:
  - Custom error codes ensure users receive clear feedback for common issues such as insufficient balance, unauthorized access, and mismatched accounts.

### Benefits:

- **Enhanced Security**: By requiring multiple signatures, the wallet prevents unauthorized or malicious transactions.
- **Decentralized Control**: Multiple parties share control of the wallet, promoting trust and cooperation.
- **Transparent Operations**: On-chain events and comprehensive error handling provide transparency and traceability.

### Use Cases:

- **DAO Treasury Management**: Organizations can manage funds collectively with member approvals for transactions.
- **Joint Investment Accounts**: Multiple users can pool resources and manage investments with a set approval threshold.
- **Escrow Services**: Securely manage and release funds with multi-party approval.

This project demonstrates how decentralized, trustless agreements and financial interactions can be facilitated on the Solana blockchain, emphasizing user collaboration and asset security.
