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
[![NPM @latest][npm-shield]][npm-url]
[![NPM @next][npm-next-shield]][npm-next-url]

# Frequency Custom RPC and Types for Polkadot JS API

An easy way to get all the custom rpc and types config to be able to easily use [Frequency](https://github.com/frequency-chain/frequency/) with the [Polkadot JS API library](https://www.npmjs.com/package/@polkadot/api) with TypeScript.

<!-- GETTING STARTED -->

## Getting Started

- `npm install @frequency-chain/api-augment` (API Augmentation Library)
- `npm install @polkadot/api` (Polkadot API Library)

## Upgrades and Matching Versions

Assuming you are using no deprecated methods, any release version should work against a release version of `@frequency-chain/api-augment`.
If you are working against a development version it is suggested that you match against the commit hash using `v0.0.0-[First 6 of the commit hash]`.

Changelog is maintained in the [releases for Frequency](https://github.com/frequency-chain/frequency/releases).

### Usage

For details on use, see the [Polkadot API library documentation](https://polkadot.js.org/docs/api).

```typescript
import { options } from "@frequency-chain/api-augment";
import { ApiPromise } from "@polkadot/api";
// ...

const api = await ApiPromise.create({
  ...options,
  // ...
});
```

<!-- CONTRIBUTING -->

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.

### Updating Definitions

Update `./definitions` if you have changed any types or RPC calls.

To generate the JS api definitions, run

```
make js
```

This command will start a Frequency node in the background and fetch API definitions from it. To stop the Frequency process, use the PID output by the command.

## Helpful Notes

### Fails to Resolve Custom RPCs

The api augmentation declares the modules used by `@polkadot/api`.
Thus the import for `@frequency-chain/api-augment` must come before any `@polkadot/api` so that the Frequency declarations resolve first.

```typescript
import { options } from "@frequency-chain/api-augment";
// Or
import "@frequency-chain/api-augment";
// Must come BEFORE any imports from @polkadot/api
import { ApiPromise } from "@polkadot/api";
```

Caches can also wreck this even if you reorder, so watch out.

- Yarn cache can sometimes cause issues (if you are using yarn): `yarn cache clear`
- Sometimes I have found blowing away the `node_modules` helps as well: `rm -Rf node_modules`

### Option<T>

Optional responses are not mapped to `null` and instead return an object with a few properties.
For more details see the [code for the Option class](https://github.com/polkadot-js/api/blob/master/packages/types-codec/src/base/Option.ts).

```javascript
const optionalExample = await api.rpc.schemas.getBySchemaId(1);
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
[npm-shield]: https://img.shields.io/npm/v/@frequency-chain/api-augment?label=npm%20%40latest&style=for-the-badge
[npm-url]: https://www.npmjs.com/package/@frequency-chain/api-augment
[npm-next-shield]: https://img.shields.io/npm/v/@frequency-chain/api-augment/next?label=npm%20%40next&style=for-the-badge
[npm-next-url]: https://www.npmjs.com/package/@frequency-chain/api-augment
