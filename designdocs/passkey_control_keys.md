# Passkey Proxy and MSA Control Key Support Design Doc

## Context and Scope

The goal of this design doc is to expand the possibilities for self-custody of MSAs via the new technology of Passkeys.
The [Passkey Proxy](./passkey_p256.md) pallet already enables token holding by Passkey holders.
Expanding this capability to the signature methods of the MSA and other pallets would open Passkeys as a method that could be used outside of the token environment.

### Why Is This Needed?

#### Expansion of Self-Custody Options for MSAs

Self-custody is difficult to use for MSAs due to the nature of the signature system.
While this expands the possibilities for non-token holding users, it decreases the self-custody.
Currently on Frequency there are only a handful of self-custody MSAs.
Making self-custody easier would expand the user control of MSAs that are part of Frequency's core mission.

## Proposal

Frequency should provide a system for Passkeys to be control keys on MSAs.

To be updated with the specific chosen details.

## Options

Note: Options are not presented in any specific order.

- [Passkey Direct](#passkey_direct)
- [Passkey Proxy Direct](#passkey_proxy_direct)
- [Wrapped Signatures](#wrapped_signatures)
- [Specific Extrinsics](#specific_extrinsics)
- [Signature Pre-registration](#signature_prereg)
- [Control Key Two-Step](#two_step)
- [MSA Proxy](#msa_proxy)

### **Passkey Direct** <a id='passkey_direct'></a>

MSAs could have direct support for the p256 addresses and signatures.
This would require either a mixed signature type or a separate set of extrinsics, not just for the MSA pallet, but other pallets that rely on signatures as well (Handles, Stateful Storage).

#### Pros
- Potentially easy, and direct Passkey signature control

#### Cons
- These direct Passkeys have some of the same issues around portability and locked website control (discussed in the [Passkey Proxy Design Doc](./passkey_p256.md)) that would undermine the goal of more self-custody and portability.

### **Passkey Proxy Direct** <a id='passkey_proxy_direct'></a>

All signature required methods should also support direct token action.
The user could just use the Passkey Proxy to act.

This would require the ability for Control Keys to use the updated address format.

#### Pros
- Passkey -> Address code already exists in the proxy

#### Cons
- The goal is that users are able to use these without tokens as well

### **Wrapped Signatures** <a id='wrapped_signatures'></a>

Create new or extend `MultiSignature` to support a signature with the components like the Passkey Proxy uses.

This expanded `MultiSignature` would have the following components:

- Passkey Public Key
- Passkey Signature
- Passkey Authenticator Data
- Passkey Client Data Json
- Account Ownership Proof Signature

The result of the verification of the signature would need to be the Account Owner address.

#### Pros
- In theory this could be used anywhere a `MultiSignature` is required. Easy use in all the existing setups across pallets.

#### Cons
- Extending `MultiSignature` is not an easy task and can have a lot of unforeseen side-effects.

### **Specific Extrinsics** <a id='specific_extrinsics'></a>

Just make a new set of extrinsics for each use case that can handle the Passkey data set.

#### Pros
- Straightforward

#### Cons
- Lots of new extrinsics
- Lots of potential bugs and issues with changes in the future

### **Signature Pre-registration** <a id='signature_prereg'></a>

Add a way for the signature to be registered with the Passkey Proxy pallet before use.
In this way, the other side need only handle the lookup on the Passkey Proxy.

#### Pros
- None

#### Cons
- This is a bad idea. It is in here as it might spur other ideas.
- The user would need tokens to register the initial signature.
- Even if it were possible via capacity, this fails to clean up and has expansive costs to maintain.
- Still requires new extrinsics or some other way to submit the data with the action.

### **Control Key Two-Step** <a id='two_step'></a>

Add (or completely switch to) a two-step control key add process.
A dual signature system requires the coordination outside of the chain.
Instead, the flow could be an "authorize" and "accept" system.

The current Control Key authorizes a new Control Key, but it does not become a Control Key until the newly authorized key submits an acceptance.
Since each key is acting independently, the action is from each key.
The signatures are validated outside of the MSA pallet.

#### Pros
- Simpler coordination of adding a new control key (of any kind).
- Might be more powerful if paired with an MSA Proxy concept.

#### Cons
- Doesn't solve the need for a Passkey-based Control Key to submit a signature for things like changing handles or other signature required MSA actions.
- Would likely require making the acceptance/rejection of an authorization either free (charging at the authorize step) or Capacity based.
- Free or Capacity transactions are not currently possible with Passkey Proxy.

### **MSA Proxy** <a id='msa_proxy'></a>

This would need its own design doc, but the core idea is that instead of having various places an MSA control key signature is required, there would be one place to call that then proxies the call with the correct authority.
This would reduce the complexity of the extrinsics and could handle multiple authorization from delegation to direct.
It could also support Token, Free, and Capacity transactions.

#### Pros
- Simplifies how someone is authorized to be an MSA.
- Easier to add other permissions and authorization methods.
- Simpler security for MSA Control Key actions

#### Cons
- Would likely still need to be combined with another option here to fully support adding control keys in this way. (See Control Key Two-Step)
- A large set of extrinsics would be deprecated.
- Should be reviewed in parallel with other design doc proposals around MSA permissions and delegations.
- Increases the cost of all signature required extrinsics as currently each is optimized for the best possible replay prevention which might not be as efficient with an MSA Proxy.

## Non-goals

Note: Some of these may become required, or fall out of requirements depending on the implementation option selected.

- Permission system for Control Keys for MSAs (separate design doc)
- Support for Ethereum Address Control Keys
- Design of an MSA Proxy and MSA Origin system to have a clear origin for the user acting

## Benefits and Risk

### Benefit: User Options

Users will have a greater options when choosing where to place their trust for their root of control of the MSA.

### Risk: Complexity

Options are good, but increase the complexity on both the Frequency side as well as the Providers who need to be able to interact with such signatures.

### Risk: P256 is Expensive

As P256 is not yet a native curve, and instead the signature verification is executed in WASM, the verification cost is much higher than `secp256k1`, `sr25519`, or `ed25519` which all have native host functions.

## Alternatives and Rationale

To be completed after the option is selected.
