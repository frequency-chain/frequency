## Context and Scope

The ability to create an on-chain account without a token to increase user accessibility for Frequency.

In Polkadot an on-chain account is created when an existential deposit is made to an address generated from cryptographic keys. In order to be able to make such deposit a user needs to acquire tokens via an exchange or transfer tokens into an account address. The process of creating an on-chain account can discourage users from using Frequency and by removing this overhead it allows Frequency to extend it services to a wider audience.

## Goals

To increase accessibility to users who want to participate in broadcasting messages without the need to acquire tokens to create an account.

In addition, the message sender must preserve the following characteristics:

- The sender of a message should be able to be identified pseudonymously.
- The sender of a message should be associated to all its messages.
- The sender of a message should be able to send messages from different devices.
- The sender of a message should be able to revoke keys associated to a device.

## Proposal

I propose a dual account system in Frequency. 

These two different types of accounts act as building blocks to facilitate the aforementioned goals. One type of account, [Token] Account, will be responsible for holding a balance, transferring tokens to other accounts, and pay certain transactions [list transactions]. Another account, Message Source Account (MSA), will be used to  broadcast messages and delegate to another MSA account.  

### [Token] Account

A token account has the ability to transfer value between accounts. More generally token accounts allow you to control funds associated to it.

Similar to Bitcoin, an account address is derived from cryptographic keys and is created by sending token to a public key. It also carries an existential deposit that keeps an account from being pruned. More on the type of supported keys below.

In Substrate a [Token] Account can be implemented by using the Balances Pallet to create a Frequency Token and storing a [Token] balance in System pallet.

### Message Source Account (MSA)

An account that can be the source of a message for itself or another message source account.

An MSA account is deterministically created with a sequential identifier called MSAId that is 8 bytes.

Unlike a token account, MSA can be created in two ways:

1. One way involves creating a [Token] Account and submitting a transaction for creating a MSA.
2. The other way is to delegate authority to another MSA.

Because of these two ways of creating an MSA it should be clear that keypairs assigned to a MSAId do not imply that these keys are associated to a Token Account. An MSA can exist independently of a Token Account.

MSAId assist in allowing users to send messages for free by delegating another MSA account. For sake of keeping the scope narrow, delegation details will be posted in a separate design doc.

![image](https://user-images.githubusercontent.com/3433442/162544133-9d163fa5-edcc-4cff-b060-9e8f4b3d9147.png)

![image](https://user-images.githubusercontent.com/3433442/162544190-cfdfb02a-ea82-4b53-9d2e-188a747a7384.png)

### Keys

A [Token] Account has an 32-byte identifier called an **AccountID**. This identifier can be derived from the public-key of the following cryptographic schemes supported by [Substrate](https://docs.substrate.io/v3/advanced/cryptography/):

- EDSA signatures using secp256k1 curve
- Ed25519 using Curve25519
- SR25519 using Ristretto group

An MSAId associated keys are also derived by the same cryptography.

## Benefits and Risks

The benefits are that it allow for users to create an MSA without the need to acquire token. Some risks are that a user has the responsibility of managing multiple keys.

## Alternatives and Rationale:

An alternative solution could be to combine both MSAId and AccountId into one identifier.

This would require accounts to be created deterministically. This could become a bottleneck for adoption as most users are accustomed an Ethereum/Bitcoin account creation flow.

Moreover, by combining an MSAId and AccountId into one identifier it limits the
ability to dissociate from an MSAId.
