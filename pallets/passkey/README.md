# Passkey Pallet

Provides a way to execute transactions using passkey signatures.

## Summary

Passkeys are a secure alternative to passwords for authentication. Due to its ease of use for the
users and having a well established support in different platforms, we decided to support Passkeys
in the Frequency chain. With this feature users would be able to execute a transactions by
authenticating themselves with a Passkey P256 signature.

### Actions

The Passkey pallet provides for:

- Executing supported transactions with a valid Passkey P256 signature

## Interactions

### Extrinsic verification
Due to lack of support for P256 signatures in the Polkadot-sdk we had to utilize an unsigned
extrinsic and implement the P256 signature all other ecosystem verifications inside
`ValidateUnsigned` trait implementation.

### Extrinsics

| Name/Description                       | Caller | Payment            | Key Events                                                                                                                                | Runtime Added |
|----------------------------------------|--------| ------------------ |-------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| `proxy`<br />Proxies an extrinisc call | Anyone   | Tokens | [`TransactionExecutionSuccess`](https://frequency-chain.github.io/frequency/pallet_passkey/module/enum.Event.html#variant.TransactionExecutionSuccess) | 92            |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_passkey/module/struct.Pallet.html) for more details.


