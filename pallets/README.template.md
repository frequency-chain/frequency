# {Name} Pallet

{Short description of the pallet}

## Summary

{What does this pallet do?}

### {Key Concept}

{Description}

### Actions

The {Name} pallet provides for:

- {Feature}
- {Feature}
- {Feature}

## Interactions

### Extrinsics

| Name/Description                 | Caller        | Payment | Key Events                                                                                                    | Runtime Added |
| -------------------------------- | ------------- | ------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `{extrinsic}`<br />{Description} | Token Account | Tokens  | [`{Event}`](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/enum.Event.html#variant.{Event}) | 1             |

See [Rust Docs](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/struct.Pallet.html) for more details.

### State Queries

| Name      | Description         | Query                    | Runtime Added |
| --------- | ------------------- | ------------------------ | ------------- |
| {Query 1} | {Query Description} | `{queryCallInCamelCase}` | 1             |

See the [Rust Docs](https://frequency-chain.github.io/frequency/{pallet_name}/pallet/storage_types/index.html) for additional state queries and details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name    | Description       | Call                                                                                                 | Node Version |
| ------- | ----------------- | ---------------------------------------------------------------------------------------------------- | ------------ |
| {RPC 1} | {RPC Description} | [`checkDelegations`]({link to the ApiServer method on https://frequency-chain.github.io/frequency/}) | v1.0.0+      |

\* Must be enabled with off-chain indexing

See [Rust Docs]({link to the ApiServer on https://frequency-chain.github.io/frequency/}) for more details.
