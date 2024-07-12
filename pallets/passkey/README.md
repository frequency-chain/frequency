# Passkey Pallet

Provides a way to execute transactions using passkey signatures.

## Summary

Passkeys are a secure alternative to passwords for authentication. Due to its ease of use for the
users and having a well established support in different platforms, Frequency chain supports passkey
p256 signatures for select transactions. With this feature users would be able to execute a
transactions by authenticating themselves with a Passkey P256 signature.

### Actions

The Passkey pallet provides for:

- Executing supported transactions with a valid Passkey P256 signature

## Interactions

### Extrinsic verification
Because the Polkadot SDK currently lacks support for P256 signatures, we had to use an unsigned
extrinsic to allow this custom verification before dispatching transactions. To achieve this, we
added P256 signature verification within the `ValidateUnsigned` trait implementation for the pallet.

Since **unsigned extrinsics** bypass verification by `SignedExtensions`, we've added the necessary
checks within the ValidateUnsigned trait implementation to mitigate potential vulnerabilities.

### Extrinsics

| Name/Description                       | Caller | Payment            | Key Events                                                                                                                                | Runtime Added |
|----------------------------------------|--------| ------------------ |-------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| `proxy`<br />Proxies an extrinsic call | Anyone   | Tokens | [`TransactionExecutionSuccess`](https://frequency-chain.github.io/frequency/pallet_passkey/module/enum.Event.html#variant.TransactionExecutionSuccess) | 92            |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_passkey/module/struct.Pallet.html) for more details.


