import { ExtDef } from '@polkadot/types/extrinsic/signedExtensions/types';
import './interfaces/types-lookup.js';
import './interfaces/augment-api.js';
import './interfaces/augment-types.js';
import './interfaces/index.js';
import * as definitions from './interfaces/definitions.js';

/**
 * Build up the types for ApiPromise.create
 */
export const types = Object.entries(definitions).reduce((acc, [_key, value]) => {
  return {
    ...acc,
    ...value.types,
  };
}, {});

/**
 * Build up the rpc calls for ApiPromise.create
 */
export const rpc = Object.entries(definitions).reduce(
  (acc, [key, value]) => {
    return {
      ...acc,
      [key]: value.rpc,
    };
  },
  // v1 rpc calls to be ignored
  {
    transactionWatch_v1: {
      submitAndWatch: {},
      unwatch: {},
    },
    transaction_v1: {
      broadcast: {},
      stop: {},
    },
    chainHead_v1: {
      body: {},
      call: {},
      continue: {},
      follow: {},
      header: {},
      stopOperation: {},
      storage: {},
      unfollow: {},
      unpin: {},
    },
  }
);

/**
 * Frequency Specific Signed Extensions
 */
export const signedExtensions: ExtDef = {
  // `CheckFreeExtrinsicUse` has no payload or extrinsic requirements
  CheckFreeExtrinsicUse: {
    extrinsic: {},
    payload: {},
  },
  HandlesSignedExtension: {
    extrinsic: {},
    payload: {},
  },
  StaleHashCheckExtension: {
    extrinsic: {},
    payload: {},
  },
  StorageWeightReclaim: {
    extrinsic: {},
    payload: {},
  },
};

/**
 * Build up all the Runtime Api Calls
 */
export const runtime = Object.entries(definitions).reduce((acc, [key, value]) => {
  return {
    ...acc,
    ...value.runtime,
  };
}, {});

/**
 * Export for easy use with Polkadot API's ApiPromise
 *
 * ```javascript
 * const api = await ApiPromise.create({
 *    ...options,
 *    provider
 *  });
 * ```
 */
export const options = {
  rpc,
  types,
  signedExtensions,
  runtime,
};
