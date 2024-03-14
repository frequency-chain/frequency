# MSA Pallet

The MSA pallet provides functionality for handling Message Source Accounts.

## Overview

The Message Source Account (MSA) is an account that can be sponsored such that public keys attached to the account
to control the MSA are not required to hold any balance, while still being able to control revocation of any delegation or control.
The MSA is represented by an Id and has one or more public keys attached to it for control.
The same public key may only be attached to ONE MSA at any single point in time.
The MSA pallet provides functions for:

- Creating, reading, updating, and deleting operations for MSAs.
- Managing delegation relationships for MSAs.
- Managing keys associated with MSA.

## Terminology

- **MSA:** Message Source Account. A Source or Provider Account for Frequency Messages. It may or may not have `Capacity`. It must have at least one public key (`AccountId`) associated with it.
  An MSA is required for sending Capacity-based messages and for creating Delegations.
- **MSA ID:** the ID number created for a new Message Source Account and associated with one or more Public Keys.
- **MSA Public Key:** the keys that control the MSA, represented by Substrate `AccountId`.
- **Delegator:** a Message Source Account that has provably delegated certain actions to a Provider, typically sending a `Message`
- **Provider:** the Message Source Account that a Delegator has delegated specific actions to.
- **Delegation:** A stored Delegator-Provider association between MSAs which permits the Provider to perform specific actions on the Delegator's behalf.

## Implementations

- [`MsaLookup`](../common_primitives/msa/trait.MsaLookup.html): Functions for accessing MSAs.
- [`MsaValidator`](../common_primitives/msa/trait.MsaValidator.html): Functions for validating MSAs.
- [`ProviderLookup`](../common_primitives/msa/trait.ProviderLookup.html): Functions for accessing Provider info.
- [`DelegationValidator`](../common_primitives/msa/trait.DelegationValidator.html): Functions for validating delegations.
- [`SchemaGrantValidator`](../common_primitives/msa/trait.SchemaGrantValidator.html): Functions for validating schema grants.

### Assumptions

- Total MSA keys should be less than the constant `Config::MSA::MaxPublicKeysPerMsa`.
- Maximum schemas, for which provider has publishing rights, be less than `Config::MSA::MaxSchemaGrantsPerDelegation`
