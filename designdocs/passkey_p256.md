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

![Registration Diagram](https://docs.google.com/drawings/d/1x9pM2OVU0zNLVJWHvHhMzLfFnIpqYU2KNVDuDMgXnrY)
![Transaction Diagram](https://docs.google.com/drawings/d/1eSgwxuCrR0x-J_7kzqnn-POXrZ5XCWM7TF0tyaxKKaw)

## 4. Data Map for Legal Teams

(Provide a data map detailing what information is stored, where it's stored, and its legal implications.)

## 5. Specification

### Passkey Registration

(Describe the process of Passkey registration, including user interactions, key generation, and backend storage.)

### Security Considerations

#### Front-end (client)
- If key generation is done in front-end, it should ideally being done inside an isolated section such as iframe or Web Worker.
- Generated Keypair should not get stored permanently (except for back up options) and removed as soon as it is not required.

#### Backend
- Passkey registration response should get verified which checks the random challenge
- Passkey Login response should get verified which checks the random challenge
- Passkey Transaction response should get verified which checks transaction related challenge.
- Any provided Frequency account signature should get verified.

#### On-chain
- Preferably using an audited crate to support p256 operations. Currently, we are using `p256` crate
which is not audited.
- If signature checks implemented **on_validate** are expensive, then this would open a vulnerability
surface for DOS attacks.

### Transaction Submission Specification

(Provide a detailed specification for forming and submitting transactions on-chain, including inputs, hashes, signature data, and necessary extensions.)

## 6. Implementation

### Backend Example Code

#### Signature verification
- Non-optimal approach
```rust
	pub fn check_passkey_signature(
		payload: &PasskeyPayload<T>,
	) -> Result<(), TransactionValidityError> {
		// deserialize to COSE key format and check the key
		let cose_key = CoseKey::from_slice(&payload.passkey_public_key[..])
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(1)))?;
		let (_, x) = cose_key
			.params
			.iter()
			.find(|(l, _)| l == &Label::Int(-2))
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(2)))?;
		let (_, y) = cose_key
			.params
			.iter()
			.find(|(l, _)| l == &Label::Int(-3))
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(3)))?;

		// convert COSE format to P256 verifying key
		let encoded_point =
			EncodedPoint::from_affine_coordinates(
				GenericArray::from_slice(&x.clone().into_bytes().map_err(|_| {
					TransactionValidityError::Invalid(InvalidTransaction::Custom(4))
				})?),
				GenericArray::from_slice(&y.clone().into_bytes().map_err(|_| {
					TransactionValidityError::Invalid(InvalidTransaction::Custom(4))
				})?),
				false,
			);
		let verify_key = p256::ecdsa::VerifyingKey::from_encoded_point(&encoded_point)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(5)))?;

		let passkey_signature =
			p256::ecdsa::DerSignature::from_bytes(&payload.passkey_signature[..])
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(6)))?;

		// extract the challenge from client_data and
		// ensure the that the challenge is the same as the call payload
		let client_data: serde_json::Value =
			serde_json::from_slice(&payload.passkey_client_data_json)
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(7)))?;
		let extracted_challenge = match client_data {
			serde_json::Value::Object(m) => {
				let challenge = m
					.get(&"challenge".to_string())
					.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(8)))?;
				if let serde_json::Value::String(base64_url_encoded) = challenge {
					let decoded = base64_url::decode(&base64_url_encoded).map_err(|_| {
						TransactionValidityError::Invalid(InvalidTransaction::Custom(9))
					})?;
					Ok(decoded)
				} else {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(10)))
				}
			},
			_ => Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(11))),
		}?;

		let encoded_payload = payload.passkey_call.encode();
		ensure!(
			encoded_payload == extracted_challenge,
			TransactionValidityError::Invalid(InvalidTransaction::Custom(12))
		);

		// prepare signing payload which is [authenticator || sha256(client_data_json)]
		let mut passkey_signature_payload = payload.passkey_authenticator.to_vec();
		passkey_signature_payload.extend_from_slice(&sha2_256(&payload.passkey_client_data_json));

		// finally verify the passkey signature against the payload
		verify_key
			.verify(&passkey_signature_payload, &passkey_signature)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadProof))
	}
```
- Some possible optimizations
  - **Compressed public key**: Currently passed public key is in **Cose** format, and it's between 70-73 bytes.
If the client can parse the Cose public key can extract the compressed encoded key. This public key can
get reduced to 33 bytes.
  - **Challenge data deduplication**: Currently the challenge data is duplicated in `expected_challenge` and
  in it's serialized format inside `passkey_client_data_json`. If the client is able to parse
  `passkey_client_data_json` and replace `challenge` field value with empty string. Then during the
  signature check we can replace that empty string with `expected_challenge` and that would allow us to
  reduce the transaction size by around **40%**. It is important that the order of the field `passkey_client_data_json`
  does not change during this operation since that would generate a different signature.

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
