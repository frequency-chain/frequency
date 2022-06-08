# Javascript Custom RPC

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


# MRC Custom RPC and Types for Polkadot JS API

An easy way to get all the custom rpc and types config to be able to easily use [MRC](https://github.com/LibertyDSNP/mrc/) with the [Polkadot JS API library](https://www.npmjs.com/package/@polkadot/api).

<!-- GETTING STARTED -->
## Getting Started

- `npm install @polkadot/api` (Polkadot API Library)
- `npm install @libertydsnp/mrc-rpc`

### Usage

For details on use, see the [Polkadot API library documentation](https://polkadot.js.org/docs/api).

```javascript
// es6 style imports
import { ApiPromise } from '@polkadot/api';
import { rpc, types } from "@libertydsnp/mrc-rpc";
// ...

const mrcAPI = await ApiPromise.create({
    // ...
    rpc,
    types,
});
```

```javascript
// commonjs require
const { ApiPromise } = require('@polkadot/api');
const { rpc, types } = require("@libertydsnp/mrc-rpc");
// ...

const mrcAPI = await ApiPromise.create({
    // ...
    rpc,
    types,
});
```

<!-- CONTRIBUTING -->
## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.


<!-- LICENSE -->
## License

Distributed under the Apache 2.0 License. See `LICENSE` for more information.


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/LibertyDSNP/mrc.svg?style=for-the-badge
[contributors-url]: https://github.com/LibertyDSNP/mrc/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/LibertyDSNP/mrc.svg?style=for-the-badge
[forks-url]: https://github.com/LibertyDSNP/mrc/network/members
[stars-shield]: https://img.shields.io/github/stars/LibertyDSNP/mrc.svg?style=for-the-badge
[stars-url]: https://github.com/LibertyDSNP/mrc/stargazers
[issues-shield]: https://img.shields.io/github/issues/LibertyDSNP/mrc.svg?style=for-the-badge
[issues-url]: https://github.com/LibertyDSNP/mrc/issues
[license-shield]: https://img.shields.io/github/license/LibertyDSNP/mrc.svg?style=for-the-badge
[license-url]: https://github.com/LibertyDSNP/mrc/blob/master/LICENSE
