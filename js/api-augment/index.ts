import { ExtDef } from "@polkadot/types/extrinsic/signedExtensions/types";
import "./interfaces/types-lookup";
import "./interfaces/augment-api";
import "./interfaces/augment-types";
import "./interfaces/index";
import * as definitions from "./interfaces/definitions";

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
export const rpc = Object.entries(definitions).reduce((acc, [key, value]) => {
  return {
    ...acc,
    [key]: value.rpc,
  };
}, {});

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
