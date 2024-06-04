# Design Doc for P256 PassKey Support

## Table of Contents

- [Design Doc for P256 PassKey Support](#design-doc-for-p256-passkey-support)
  - [Table of Contents](#table-of-contents)
  - [1. Introduction](#1-introduction)
  - [2. Terminology](#2-terminology)
    - [Accounts](#accounts)
    - [Keys](#keys)
    - [Signatures](#signatures)
  - [3. Data Flow Diagram](#3-data-flow-diagram)
  - [4. Data Map for Legal Teams](#4-data-map-for-legal-teams)
  - [5. Specification](#5-specification)
    - [Passkey Registration](#passkey-registration)
    - [Security Considerations](#security-considerations)
    - [Transaction Submission Specification](#transaction-submission-specification)
  - [6. Implementation](#6-implementation)
    - [Backend Example Code](#backend-example-code)
    - [Frontend Example Code](#frontend-example-code)
  - [7. Options for Discussion](#7-options-for-discussion)
    - [Unsigned Extensions vs MultiSignature](#unsigned-extensions-vs-multisignature)
    - [Generic Key Support](#generic-key-support)
    - [Separate Pallet](#separate-pallet)
  - [8. Conclusion](#8-conclusion)

## 1. Introduction

This document outlines the design considerations and specifications for integrating P256 Passkey support for performing transactions on Frequency chain. Passkey support aims to provide a novel non custodial solution for managing user accounts and signing transactions on-chain.

## 2. Terminology

### Accounts

- **Authorized Account**: The user account associated with the Passkey.
- **Parent Account**: The account holding the primary private key for transaction signing.
- **Frequency Access**: The platform facilitating the management and interaction with user accounts.

### Keys

- **Passkey**: P256 key pair used for transaction signing and account management. This is the primary key used for transaction signing.
- **Passkey Public Key**: The public key derived from the Passkey private key.
- **Seed Phrase**: A mnemonic phrase used to generate cryptographic keys, particularly for SR25519 accounts. This is used for signing passkey proving account ownership.
- **Seed Public Key**: The public key derived from the Seed Phrase.

### Signatures

- **Seed signature on PassKey Public Key**: A cryptographic signature generated using the Seed Phrase private key. The data being signed is public key derived from passkey. This is used to prove ownership of parent account.
- **Passkey Signature**: A cryptographic signature generated using the Passkey private key. This is presented to passkey enabled services as a challenge-response mechanism. Passkeys are used to generate two signatures as follows:
  - **Signature on Seed Public Key**: Passkey signs a message containing the Seed Public Key. This signature is retained by the Frequency Access platform and/or maybe used for account recovery.
  - **Signature on Transactions**: Passkey signs the transaction payload which needs to be submitted on-chain. This signature is used to verify the authenticity of the transaction.

## 3. Data Flow Diagram

![Data Flow Diagram](insert_diagram_link_here)

## 4. Data Map for Legal Teams

(Provide a data map detailing what information is stored, where it's stored, and its legal implications.)

## 5. Specification

### Passkey Registration

(Describe the process of Passkey registration, including user interactions, key generation, and backend storage.)

### Security Considerations

(Outline the security measures to be implemented both on the client-side and server-side to ensure the safety of user data and transactions.)

### Transaction Submission Specification

(Provide a detailed specification for forming and submitting transactions on-chain, including inputs, hashes, signature data, and necessary extensions.)

## 6. Implementation

### Backend Example Code

(Include code snippets demonstrating how to implement Passkey support on the backend.)

### Frontend Example Code

(Include code snippets demonstrating how to implement Passkey support on the frontend.)

## 7. Options for Discussion

### Unsigned Extensions vs Extending MultiSignature

#### Unsigned Extensions
In this variant we will have an unsigned extrinsic and all the related checks would be done inside
`ValidateUnsigned` trait implementation for the pallet.

##### Pros/Cons
- **Pro**: Faster and already proven implementation
- **Pro**: Flexibility to be replaced with other implementations
- **Con**: Some duplication of code between existing checks on signed extensions and the checks
 implemented on `ValidateUnsigned`
- **Con**: An unsigned extrinsic implementation might open up a new unknown attack vector.

#### Extending MultiSignature
In this variant we will extend `MultiSignature` enum and replace it with a new enum which will have
a new `P256` signature type.

##### Pros/Cons
- **Pro**: No need to use unsigned extrinsic and all extra checks would be done inside a new signed
extension.
- **Pro**: A uniform and generic solution which would allow having P256 signature scheme to be used
for other operations on chain.
- **Pro**: This would allow us to use P256 accounts to hold token (but that might not be desirable)
- **Pro**: We could use the P256 keys as MSA control keys.
- **Con**: Requires significant effort (in case if no hard constraints detected) to implement compared to using
unsigned extension and once deployed there would not be an easy way for a backwards compatible rollback.
Here is a quick breakdown for known issues:
  - Signature size mismatch force us to implement a new `Signature` type with all required traits.
  - Public key size mismatch might force us to implement a new `Publickey` type with all required traits.
  - Reimplementing `MultiSignature`, `MultiSigner` and other types with `P265` functionality added.
  - Adding signature and key generation support on polkadotJS and all frontend implementations
  - Might require DB migration for already stored MultiSignatures


### Generic Key Support
There is an issue with the `account_key` generation flow since it would only support transactions
that do not require any Msa account. To be able to use the PassKey feature for the majority of
transactions, it might be better if the `account_key` was already in a wallet and the Msa account
was created for that key,and we register a passkey using the same.

### Separate Pallet
_Question_: Should we implement this feature in a separate pallet or just use already existing `frequency-tx-payment`
pallet?

One argument against having it in a separate pallet is since there is no extra data required to be
stored on-chain, it seems less necessary to split it into a separate pallet.
Another argument in favor of having it in a new pallet is to be able to share this pallet with other
para-chains in the Polkadot ecosystem.

## 8. Conclusion

(Summarize the key points of the design document and outline next steps for implementation and further discussion.)
