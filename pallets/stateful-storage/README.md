# Stateful Storage Pallet

The Stateful Storage pallet provides per-MSA functionality for reading and writing stateful data (where only the latest state is relevant).

## Summary

When a Schema uses `Paginated` or `Itemized`, the payload data is stored in this pallet.
Data stored is user-centric instead of time-centric as with the Messages Pallet.
The pallet storage uses [child storage](https://paritytech.github.io/polkadot-sdk/master/frame_support/storage/child/index.html) making direct query access complex.
Custom RPCs are provided for easy access to data.

### Paginated Data (`PayloadLocation:Paginated`)

Data is stored in multiple pages, each `1_024` bytes in size (as defined by `constants::MaxPaginatedPageSizeBytes`).
Each page contains a single item of the associated schema.
Page count is limited to `33` per Schema Id, though there may be holes in that range (limit defined by `constants::MaxPaginatedPageId`).
This is most useful for schemas with a larger per-item size and smaller potential item count.

### Itemized Data (`PayloadLocation:Itemized`)

Data is stored in a single page with a max size of `10_240` bytes (defined by `constants::MaxItemizedPageSizeBytes`).
The page contains multiple items of the associated schema.
The maximum size of each items is `1_024` bytes (defined by `constants::MaxItemizedBlobSizeBytes`) .
This is most useful for schemas with a relatively small item size and higher potential item count.
The read and write complexity is O(n) when n is the number of bytes for all items.



### Actions

The Stateful Storage pallet provides for:

- Per MSA and Schema storage of stateful data
- Read/write access storage cost limited to a single MSA Id and Schema Id pair
- A high write throughput
- A high read throughput
- Data race condition protection


## Interactions

### Extrinsics

| Name/Description                 | Caller        | Payment | Key Events                                                                                                    | Runtime Added |
| -------------------------------- | ------------- | ------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `apply_item_actions`<br />Applies a set of actions to an itemized storage array | Provider or MSA Owner | Capacity or Tokens  | [`ItemizedPageUpdated`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.ItemizedPageUpdated)<br />[`ItemizedPageDeleted`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.ItemizedPageDeleted) | 22             |
| `apply_item_actions_with_signature_v2`<br />Applies a set of actions to an itemized storage array  with a signature authorization | Provider or MSA Owner | Capacity or Tokens  | [`ItemizedPageUpdated`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.ItemizedPageUpdated)<br />[`ItemizedPageDeleted`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.ItemizedPageDeleted) | 45             |
| `upsert_page`<br />Sets the data for a specific page index | Provider or MSA Owner | Capacity or Tokens  | [`PaginatedPageUpdated`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.PaginatedPageUpdated) | 22             |
| `delete_page`<br />Deletes a specific page index | Provider or MSA Owner | Capacity or Tokens  | [`PaginatedPageDeleted`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.PaginatedPageDeleted)| 22             |
| `upsert_page_with_signature_v2`<br />Sets the data for a specific page index with a signature authorization | Provider or MSA Owner | Capacity or Tokens  | [`PaginatedPageUpdated`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.PaginatedPageUpdated) | 46             |
| `delete_page_with_signature_v2`<br />Deletes a specific page index with a signature authorization | Provider or MSA Owner | Capacity or Tokens  | [`PaginatedPageDeleted`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.PaginatedPageDeleted)| 46             |

See [Rust Docs](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/struct.Pallet.html) for more details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name    | Description       | Call                                                                                                 | Node Version |
| ------- | ----------------- | ---------------------------------------------------------------------------------------------------- | ------------ |
| Get Paginated Storage | Retrieves the paginated storage for the given MSA Id and Schema Id | [`getPaginatedStorage`](https://frequency-chain.github.io/frequency/pallet_stateful_storage_rpc/trait.StatefulStorageApiServer.html#tymethod.get_paginated_storage) | v1.4.0+      |
| Get Itemized Storage | Retrieves the itemized storage for the given MSA Id and Schema Id | [`getItemizedStorage`](https://frequency-chain.github.io/frequency/pallet_stateful_storage_rpc/trait.StatefulStorageApiServer.html#tymethod.get_itemized_storage) | v1.4.0+      |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_stateful_storage_rpc/trait.StatefulStorageApiServer.html) for more details.
