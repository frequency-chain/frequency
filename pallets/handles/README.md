# Handles Pallet

Creates human-readable, homoglyph-attack resistant handles for MSAs.

## Summary

Provides MSAs with an optional, but unique handle.

A handle consists of:
- **Base Handle:** The user's chosen handle. It is *not* guaranteed to be unique without the suffix. It is linked to a normalized version for Handle to MSA Id resolution. See [Normalization Details](#normalization-details) below.
- **Suffix:** The suffix is a numeric value appended to the user's base handle to ensure the display handle (base handle + suffix) is unique.
- **Display Handle:** The user's original (un-normalized, but with whitespace trimmed and consolidated) base handle string and the suffix together (`base`.`suffix`) constitute a unique identifier for a user.

### Suffixes

In order to allow multiple users to select the same base handle, a unique numeric suffix is appended to the Base Handle to form the Display Handle.
The suffix is generated from a random sequence such that each suffix is unique based on the normalized version of the handle.
For example, if there are two users who choose the handle `alice`, one would be `alice.57` and the other `alice.84`.

## Normalization Details

For safety, user handles are normalized for lookup purposes. An end user can therefore be reasonably assured that a display handle with the correct numeric suffix resolves to the desired user, regardless of the variant of the displayed base. (ie, for the suffix `.25`, all variants of the normalized base `a1ice` resolve to the same user: `a1ice`, `alice`, `alicë`, `a1icé`, etc...)


### Character Normalization

Normalization removes diacriticals and converts to the lowercase version of the character.
For example, `Zaë` will be normalized `zae`.

### Homoglyph Normalization

Two or more characters that appear the same to the user are [homoglyphs](https://en.wikipedia.org/wiki/Homoglyph).
To prevent most homoglyph attacks where one user attempts to impersonate another, the normalization converts all known homoglyphs to a single character.
Thus, any version that normalizes to the same value are considered to be the same.
For example, for the suffix `.25`, all variants of the normalized base `a1ice` resolve to the same user: `a1ice`, `alice`, `alicë`, `a1icé`, etc...

## Handle Requirements

To programmatically check if a handle is valid, see the [`validate_handle` RPC](#RPCs).

### Pre-Normalization Validation

- MUST be UTF-8
- MUST NOT be more than 26 bytes
- MUST not contain one of the blocked characters: ``" # % ( ) , . / : ; < > @ \ ` { }``

### Post-Normalization Validation

- MUST have a character length of at least 3 and no more than 20
- MUST not be a reserved word or a [homoglyph](#homoglyph-normalization) of it:
  - `adm1n` (`admin`)
  - `every0ne` (`everyone`)
  - `a11` (`all`)
  - `adm1n1strat0r` (`administrator`)
  - `m0d` (`mod`)
  - `m0derat0r` (`moderator`)
  - `here` (`here`)
  - `channe1` (`channel`)
- MUST only contain characters from the allowed unicode ranges (See [`ALLOWED_UNICODE_CHARACTER_RANGES`](https://github.com/frequency-chain/frequency/blob/main/pallets/handles/src/handles-utils/constants.rs) for the full list)

### Actions

The Handles pallet provides for:

- Unique identifier for an MSA
- Creation by proxy
- Tokenless handle removal
- Shuffled sequences for Suffixes

## Interactions

### Extrinsics

| Name/Description                 | Caller        | Payment | Key Events                                                                                                    | Runtime Added |
| -------------------------------- | ------------- | ------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `claim_handle`<br />Claim a handle with the given  | Provider or MSA Owner | Capacity or Tokens  | [`HandleClaimed`](https://frequency-chain.github.io/frequency/pallet_handles/pallet/enum.Event.html#variant.HandleClaimed) | 27             |
| `retire_handle`<br />Retire a handle. Retired handles + suffix are never reused.   | MSA Owner | Free  | [`HandleRetired`](https://frequency-chain.github.io/frequency/pallet_handles/pallet/enum.Event.html#variant.HandleRetired) | 27             |
| `change_handle`<br />Convenience method to retire and then claim a new handle  | Provider or MSA Owner | Capacity or Tokens  | [`HandleRetired`](https://frequency-chain.github.io/frequency/pallet_handles/pallet/enum.Event.html#variant.HandleRetired), [`HandleClaimed`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.HandleClaimed) | 47             |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_handles/pallet/struct.Pallet.html) for more details.

### State Queries

Note: RPC use is suggested over the direct state queries for handles.

| Name      | Description         | Query                    | Runtime Added |
| --------- | ------------------- | ------------------------ | ------------- |
| Get Handle by MSA Id  | Returns the Display Handle and the block number in which it was claimed   | `msaIdToDisplayName` | 29             |
| Get MSA Id by Canonical Base and Suffix  | Uses the stored canonical lookup string NOT the display handle with the suffix to retrieve the MSA Id   | `canonicalBaseHandleAndSuffixToMSAId` | 29             |

See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_handles/pallet/storage_types/index.html) for additional state queries and details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name    | Description       | Call                                                                                                 | Node Version |
| ------- | ----------------- | ---------------------------------------------------------------------------------------------------- | ------------ |
| Get Handle by MSA Id | Returns the base handle and suffix as well as the canonical version of the handle | [`getHandleforMSA`](https://frequency-chain.github.io/frequency/pallet_handles_rpc/trait.HandlesApiServer.html#tymethod.get_handle_for_msa) | v1.6.0+      |
| Get MSA Id by Display Handle | Returns the MSA Id for a given Display Handle | [`getMsaForHandle`](https://frequency-chain.github.io/frequency/pallet_handles_rpc/trait.HandlesApiServer.html#tymethod.get_handle_for_msa) | v1.6.0+      |
| Validate Handle String | Checks to see if the handle string validates | [`validateHandle`](https://frequency-chain.github.io/frequency/pallet_handles_rpc/trait.HandlesApiServer.html#tymethod.validate_handle) | v1.8.0+      |
| Get Next Suffixes | Given a Base Handle and count, returns the next suffixes that will be used for claimed handles | [`getNextSuffixes`](https://frequency-chain.github.io/frequency/pallet_handles_rpc/trait.HandlesApiServer.html#tymethod.get_next_suffixes) | v1.8.0+      |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_handles_rpc/trait.HandlesApiServer.html) for more details.
