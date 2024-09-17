# Handles Pallet

Creates human-readable, homoglyph-attack resistant handles for MSAs.

## Summary

Provides MSAs with an optional, but unique handle.

A handle consists of:
- **Base Handle:** The user's chosen handle. It is *not* guaranteed to be unique without the suffix. It is linked to a normalized version for Handle to MSA Id resolution. See [UTF-8 Support](#utf-8-support) and [Homoglyph Attack Resistence](#homoglyph-attack-resistence) below.
- **Suffix:** A suffix is a unique numeric value appended to a user's base handle to make it unique.
- **Display Handle:** The user's original (un-normalized, but with whitespace trimmed and concatenated) base handle string and the suffix together (`base`.`suffix`) constitute a unique identifier for a user.

### UTF-8 Support

Handles are able to have many allowed ranges of UTF-8.
Some ranges, such as emoji, are currently not allowed.
Due to the handling of homoglyphs, some handles will resolve to the same value.
For example, while the display may have diacriticals or homoglyphs, the handle is stored with diacriticals and homoglyphs normalized.
So `Zoë.35` and `Zoe.35` will both resolve to the same MSA Id.

### Homoglyph Attack Resistance

Two or more characters that appear the same to the user are [homoglyphs](https://en.wikipedia.org/wiki/Homoglyph).
To prevent most homoglyph attacks where one user attempts to impersonate another, the user's requested Base Handle is converted to a canonical, normalized version of the handle.
The canonical version determines the suffix series that is chosen.
An end user can therefore be reasonably assured that a display handle with the correct numeric suffix resolves to the desired user, regardless of the homoglyph-variant of the displayed base. (ie, for the suffix `.25`, all variants of the canonical base `a1ice` resolve to the same user: `a1ice`, `alice`, `alicë`, `a1icé`, etc...)


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
