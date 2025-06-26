# Recovery SDK package

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![NPM @latest][npm-shield]][npm-url]
[![NPM @next][npm-next-shield]][npm-next-url]

# Recovery SDK for Frequency Recovery System

Support for generating and validating the hashes needed to use the Recovery System on Frequency.

<!-- GETTING STARTED -->

## Getting Started

- `npm install @frequency-chain/recovery-sdk`

## Usage

| Method | Description |
| --- | --- |
| `generateRecoverySecret` | Generates a new Recovery Secret, formatted for users |
| `getRecoveryCommitment` | Gets the Recovery Commitment for the given Recovery Secret and contact |
| `getIntermediaryHashes` | Gets the Intermediary Recovery Hashes for the Recovery Provider to use to recover |
| `getRecoveryCommitmentFromIntermediary` | Gets the Recovery Commitment from the Intermediary Recovery Hashes |

## Upgrades and Matching Versions

`@frequency-chain/recovery-sdk` is versioned with Frequency and should work against any release version of a Frequency node that has a compatible recovery system.
If you are working against a development node that has changes in the recovery system, you should match the package version against the commit hash using `v0.0.0-[First 6 of the commit hash]`.

Changelog is maintained in the [releases for Frequency](https://github.com/frequency-chain/frequency/releases).


<!-- CONTRIBUTING -->

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.

<!-- LICENSE -->

## License

Distributed under the Apache 2.0 License. See `LICENSE` for more information.

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/frequency-chain/frequency.svg?style=for-the-badge
[contributors-url]: https://github.com/frequency-chain/frequency/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/frequency-chain/frequency.svg?style=for-the-badge
[forks-url]: https://github.com/frequency-chain/frequency/network/members
[stars-shield]: https://img.shields.io/github/stars/frequency-chain/frequency.svg?style=for-the-badge
[stars-url]: https://github.com/frequency-chain/frequency/stargazers
[issues-shield]: https://img.shields.io/github/issues/frequency-chain/frequency.svg?style=for-the-badge
[issues-url]: https://github.com/frequency-chain/frequency/issues
[license-shield]: https://img.shields.io/github/license/frequency-chain/frequency.svg?style=for-the-badge
[license-url]: https://github.com/frequency-chain/frequency/blob/master/LICENSE
[npm-shield]: https://img.shields.io/npm/v/@frequency-chain/recovery-sdk?label=npm%20%40latest&style=for-the-badge
[npm-url]: https://www.npmjs.com/package/@frequency-chain/recovery-sdk
[npm-next-shield]: https://img.shields.io/npm/v/@frequency-chain/recovery-sdk/next?label=npm%20%40next&style=for-the-badge
[npm-next-url]: https://www.npmjs.com/package/@frequency-chain/recovery-sdk
