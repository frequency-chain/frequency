# Provider Contexts Design Doc

## Context and Scope

On Frequency, a Provider may either represent an application or a company.
This document will outline the changes that will allow a Provider to represent a company with one or multiple applications.
It also enables social wallet providers, custodial and non-custodial, to provide safe application information to their users.

This is an expansion of the [Registered Provider](https://github.com/frequency-chain/frequency/blob/main/designdocs/provider_registration.md) concepts.

Reference Document: [SIWF Specification](https://projectlibertylabs.github.io/siwf/v2/docs/)

### Why Is This Needed?

#### Concept of an Application Separate from a Provider

Many companies have a single major product, but some companies have multiple applications that serve different needs.
While each application could be a separate provider, this causes two issues: The sharing of capacity and the user permissions.

Having separate providers would require a single company to have separate Capacity staking for each application.
As each these applications are part of the same company, it is reasonable to say that those applications should share Capacity as a single legal entity.

User permissions are primarily write-based at this time, however it is critical that users understand the delegation and their data is shared at a company level, no matter the chain representation of such.
To align with the data security model of company-to-user in the world today, permissions, and thus Provider-level delegations continue to reflect the reality best.

Finally, this structure does not prevent the setup of a 1:1 relationship between Application and Provider.
If a company wishes, they can create more than one Provider for the purposes of separation.
This may be desirable if there is a subsidiary company or other complex structures.

#### User Trust in Application Presentation

Users need to know what application they are logging into and trust that the displayed information is not a phishing attack.
Users have a reasonable expectation that the chain and wallet provide a level of protection against phishing attacks from Providers and Applications.

#### Easy Wallet Integration of Data

Wallets displaying login request information need to be able to easily obtain and verify the information they want to show the user.

## Proposal

Frequency should provide a safeguarded system of setting and updating Provider and Provider Application Contexts for wallets to display to users.

- [Storage and Structure of the Data](#data)
- [Mainnet Approval Flow](#governance)
- [Example of Wallet Usage via SIWF](#siwf)
- [Provider Dashboard Steps](#dashboard)

### **Storage and Structure of the Data** <a id='data'></a>

Currently, the `ProviderRegistryEntry` has only a `name` property, and no way to update said name.

Applications have two different pieces of default and (potentially) internationalized data:
- name
- logo/image

Limits
1. Provide a space to internationalize up to 150 localizations (Windows and macOS each have < 150).
2. Image MUST BE encoded (`png`)[https://www.w3.org/TR/png-3/] that is `250x100` (support for future sizes and light/dark should be considered).

The data structure must support both of these and internationalization.

Proposed are the following changes:

1. `ProviderRegistryEntry` struct be renamed `ApplicationRegistryEntry`
2. `ApplicationRegistryEntry` have the following properties:
    ```rust
    pub struct ApplicationRegistryEntry<T, U>
    where
        T: Get<u32>,
        U: Get<u32>,
    {
        pub default_name: BoundedVec<u8, T>,
        pub localized_names: BTreeMap<Vec<u8>, BoundedVec<u8, T>>,
        pub default_logo_250_100_png_bytes: BoundedVec<u8, U>,
        pub localized_logo_250_100_png_bytes: BTreeMap<Vec<u8>, BoundedVec<u8, U>>,
    }
    ```
3. The `ProviderToRegistryEntry` be updated to use `ApplicationRegistryEntry` so that the Provider has a "default" `ApplicationRegistryEntry`.
    ```rust
    #[pallet::storage]
	pub type ProviderToRegistryEntry<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ProviderId,
		ApplicationRegistryEntry<T::MaxProviderNameSize, T::MaxProviderLogo250X100Size>,
		OptionQuery,
	>;
    ```
4. New `ProviderToApplicationRegistryEntry` storage be initialized:
    ```rust
    // Alias for clarity
    type ApplicationIdentifier<T: Config> = BoundedVec<u8, T::MaxProviderNameSize>

    #[pallet::storage]
	pub type ProviderToApplicationRegistryEntry<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ProviderId,
		Twox64Concat,
		ApplicationIdentifier<T>,
		ApplicationRegistryEntry<T::MaxProviderNameSize, T::MaxProviderLogo250X100Size>,
		OptionQuery,
	>;
	```
5. `MaxProviderNameSize` be increased to `256`.
6. `MaxProviderLogo250X100Size` be created and the limit set to `131_072` (128 KiB).

### **Mainnet Approval Flow** <a id='governance'></a>

_Any_ change to the Provider or Application Context must be approved by governance.
For now, that governance approval is any single Frequency Council member.

```mermaid
sequenceDiagram
    participant P as Provider
    participant D as Provider Dashboard
    participant F as Frequency
    participant G as Frequency Council

    note over P,G: Provider Registration
    P->>D: Login
    D-->>+F: Create MSA
    F-->>-D: MSA created
    P->>D: Request to be Provider form
    P->>D: Add Applications as needed
    D-->>+F: Generate Frequency Council Proposal
    G->>F: Review and Approve Proposal
    F->>F: Execute Provider registration
    F-->>-D: MSA is a Provider

    note over P,G: Provider Update Registration
    P->>D: Login
    P->>D: See Provider and Application Information
    P->>D: Request to update Provider form
    P->>D: Update/add Applications as needed
    D-->>+F: Generate Frequency Council Proposal
    G->>F: Review and Approve Update Proposal
    F->>F: Execute Provider changes as an upsert
    F-->>-D: Update display
    P->>D: See Updated Provider and Application Information
```

#### Open Questions
- Is this a set or edit pattern for the applications?
- Should adding/updating applications be a separate call?
- Should adding/updating a translation be a separate call?

#### Implementation Suggestion

To limit the amount of unapproved bytes intended to be interpreted as images on chain, an option would be to merely submit the hashes of images for approval.
This hash would then be allowlisted for that provider.
Once approved, that provider could submit the image to chain.
In the governance process, a link to the image would need to be submitted.

### **Example of Wallet Usage via SIWF** <a id='siwf'></a>

SIWF [Signed Request Payload](https://projectlibertylabs.github.io/siwf/v2/docs/DataStructures/All.html) can expand the optional `applicationContext` value with a new optional `id` field that is the text of the `ApplicationIdentifier` on chain.
This would allow a smooth transition between `applicationContext.url` and `applicationContext.id` with both being optional.

The Wallet would then:
1. Verify the SIWF Signed Request.
2. Lookup the Provider via the `publicKey` in the SIWF Signed Request.
3. If any, fetch the Application Identifier from Frequency.
4. Display the information from the `ApplicationRegistryEntry` (or the Provider Registry Entry `ProviderToRegistryEntry` if there is no application context identifier) to the user to help them know who they are authorizing.
5. Allow the user to continue the login process.

### **Provider Dashboard Steps** <a id='dashboard'></a>

Provider Dashboard needs to be able to:

- Create a provider without any application context other than the default provider context
- Update the default provider context, logos, and translations
- Add new Application Contexts with a identifier (unique to that provider), logos, and translations
- Remove an existing Application Context
- Update an Application Contexts with new logos, and translations


## Non-goals

- Implementation of verified credentials for applications (Aka FooBar is approved by <Consumer Org>)
- Providing for independent body to perform verification

## Benefits and Risk

### Benefit: User protection and Application diversity

Users will have a greater confidence when they are logging into an application that the application is represented honestly, even with the continuous risk of trusting any new application with user data.

### Risk: Initial structure of only one Frequency Council member approving changes in an application

Tricking a single member is much simpler than a more detailed vetting process; however, at this stage, the number of providers is small.

This process should be re-evaluated as the number of Providers grows.

### Risk: Logo Images on Chain

This provides a direct way to place image content on chain, intended to be interpreted as image content.
While this must still pass through the governance step to be used by others via the content the image would still be in the chain history as there is always a risk of problematic images being proposed.
This risk is mitigated by:
- A registered provider must be taking the action
- Larger images require larger token fees to cover the cost
- IF applied, the suggested hash requirement for the image upload could remove this issue entirely

The mitigation is enough for the benefit of clear branding for users to outweigh this risk.

## Alternatives and Rationale

1. Trusted Domains
2. Off-chain authentication and verification

### Verify and Just Trust a Domain

While using the domain name system for verification is fine, the issue remains that a malicious Provider could still portray themselves as another application via a Phishing attack.
The desired trust is that the Provider is not maliciously presenting themselves, not that the domain is correct according to the user which would prevent phishing.

Example phishing attack: Foobar.co could represent themselves as Foobar.com and display the logo of Foobar.com from `https://foobar.co/images/logo.png`.

### Off-chain Authentication and Verification

The wallet interface could also (via some mechanism) reach out for the verification or have an allowlist of verified providers.

However, this either requires a centralized service or a patchwork of verifications.
Self-verification is not possible due to the phishing attack.
There is a coordination service of Frequency, but using a schema or other non-verified setup would still require reaching out and extending trust to some other 3rd-party or patchwork verification system.

In the end this solution results in these two problems:
- 3rd-party patchwork systems increase friction and complexity.
- External centralization is undesirable.

There is a future where 3rd-party verification can be used in conjunction with on-chain approval.
