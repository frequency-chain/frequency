# Static Schemas package

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

# Static schemas

A convenient way to get schema details locally with TypeScript.

<!-- GETTING STARTED -->

## Getting Started

- `npm install @frequency-chain/schemas` (static schemas library)

## Usage
After importing, any of the following **Maps** can be used to fetch desired schema information.

- `ID_TO_SCHEMA_FULL_NAME` is a **Map** that returns full names from schema ids (example `dsnp.tombstone@v1`)
- `FULL_NAME_TO_ID` is a **Map** that returns schema id from full name
- `ID_TO_SCHEMA_INFO` is a **Map** that return `SchemaInfo` from schema id

Here is an example of a schema info object

```javascript
  {
    id: 7,
    namespace: 'dsnp',
    name: 'public-key-key-agreement',
    version: 1,
    deprecated: false,
    modelType: 'AvroBinary',
    payloadLocation: 'Itemized',
    appendOnly: true,
    signatureRequired: true,
  },
```

## Upgrades and Matching Versions

Assuming you are using no deprecated methods, any release version should work against a release version of `@frequency-chain/schemas`.
If you are working against a development version it is suggested that you match against the commit hash using `v0.0.0-[First 6 of the commit hash]`.

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
[npm-shield]: https://img.shields.io/npm/v/@frequency-chain/schemas?label=npm%20%40latest&style=for-the-badge
[npm-url]: https://www.npmjs.com/package/@frequency-chain/schemas
[npm-next-shield]: https://img.shields.io/npm/v/@frequency-chain/schemas/next?label=npm%20%40next&style=for-the-badge
[npm-next-url]: https://www.npmjs.com/package/@frequency-chain/schemas
