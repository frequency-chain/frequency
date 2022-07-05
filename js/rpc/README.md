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


# Frequency Custom RPC and Types for Polkadot JS API

An easy way to get all the custom rpc and types config to be able to easily use [Frequency](https://github.com/LibertyDSNP/frequency/) with the [Polkadot JS API library](https://www.npmjs.com/package/@polkadot/api).

<!-- GETTING STARTED -->
## Getting Started

- `npm install @polkadot/api` (Polkadot API Library)
- `npm install @dsnp/frequency-rpc`

### Usage

For details on use, see the [Polkadot API library documentation](https://polkadot.js.org/docs/api).

```javascript
// es6 style imports
import { ApiPromise } from '@polkadot/api';
import { rpc, types } from "@dsnp/frequency-rpc";
// ...

const frequencyAPI = await ApiPromise.create({
    // ...
    rpc,
    types,
});
```

```javascript
// commonjs require
const { ApiPromise } = require('@polkadot/api');
const { rpc, types } = require("@dsnp/frequency-rpc");
// ...

const frequencyAPI = await ApiPromise.create({
    // ...
    rpc,
    types,
});
```

<!-- CONTRIBUTING -->
## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.


## Helpful Notes

### Option<T>

Optional responses are not mapped to `null` and instead return an object with a few properties.
For more details see the [code for the Option class](https://github.com/polkadot-js/api/blob/master/packages/types-codec/src/base/Option.ts).
```javascript
const optionalExample = await api.rpc.msa.getMsaId(account);
// Does the Option have a value?
if (!optionalExample.isEmpty) {
    // Get the value
    return optionalExample.value;
}
return null;
```

### Vec<T>

Vector responses are not mapped directly to a JavaScript Array.
Instead they are mapped to the [Vec class](https://github.com/polkadot-js/api/blob/master/packages/types-codec/src/base/Vec.ts) which does extend [Array](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array).
Thus, you can still use `map`, `forEach`, etc... with responses or access the values directing via `.values()`.

<!-- LICENSE -->
## License

Distributed under the Apache 2.0 License. See `LICENSE` for more information.


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/LibertyDSNP/frequency.svg?style=for-the-badge
[contributors-url]: https://github.com/LibertyDSNP/frequency/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/LibertyDSNP/frequency.svg?style=for-the-badge
[forks-url]: https://github.com/LibertyDSNP/frequency/network/members
[stars-shield]: https://img.shields.io/github/stars/LibertyDSNP/frequency.svg?style=for-the-badge
[stars-url]: https://github.com/LibertyDSNP/frequency/stargazers
[issues-shield]: https://img.shields.io/github/issues/LibertyDSNP/frequency.svg?style=for-the-badge
[issues-url]: https://github.com/LibertyDSNP/frequency/issues
[license-shield]: https://img.shields.io/github/license/LibertyDSNP/frequency.svg?style=for-the-badge
[license-url]: https://github.com/LibertyDSNP/frequency/blob/master/LICENSE
