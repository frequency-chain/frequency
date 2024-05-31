# Frequency Transaction Pallet

Allows users to perform transactions using Capacity.

## Summary

Frequency supports the following alternative payments:

- Capacity: A refillable resource limited to a subset of transactions.

The Frequency Transaction Pallet proxies or nests one or more calls inside of the root call to enable them to be paid using an alternative method.
For example, to call something like `add_ipfs_message(params)` with Capacity, one would call `pay_with_capacity(add_ipfs_message(params))`.
The `pay_with_capacity` will verify that the inner call is allowed with capacity.

### Requirements for Paying with Capacity

The account must:
1. Be a current control key on a Provider.
2. Have a minimum balance of the existential deposit.

### Capacity Stable Weights

Token costs on a specific transaction can fluctuate due to benchmark changes, but the costs of capacity calls are setup to be relatively consistent.
This consistency is because the price of a capacity transaction is use a separate set of benchmark weights, which are kept constant at a given point in time.
While periodic updates do occur, the costs typically stay the same or may even decrease over time.

### Actions

The Frequency Transaction pallet provides for:

- Transacting using only Capacity

## Interactions

### Extrinsics

| Name/Description                 | Caller        | Payment | Key Events                                                                                                    | Runtime Added |
| -------------------------------- | ------------- | ------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `pay_with_capacity`<br />Proxies a single capacity allowed call  | Provider | Capacity  | [`CapacityWithdrawn`](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/enum.Event.html#variant.CapacityWithdrawn)* | 1             |
| `pay_with_capacity_batch_all`<br />Proxies a batch (limit 10) of capacity allowed calls  | Provider | Capacity  | [`CapacityWithdrawn`](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/enum.Event.html#variant.CapacityWithdrawn)* | 1             |

\* Note: This is just the event noting the use of Capacity. Additional events for the call being proxied will still occur.

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_frequency_tx_payment/pallet/struct.Pallet.html) for more details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name    | Description       | Call                                                                                                 | Node Version |
| ------- | ----------------- | ---------------------------------------------------------------------------------------------------- | ------------ |
| Compute Capacity Fee | Calculates the expected Capacity cost of the supplied transaction | [`computeCapacityFeeDetails`](https://frequency-chain.github.io/frequency/pallet_frequency_tx_payment_rpc/trait.CapacityPaymentApiServer.html#tymethod.compute_capacity_fee_details) | v1.8.0+      |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_frequency_tx_payment_rpc/trait.CapacityPaymentApiServer.html) for more details.
