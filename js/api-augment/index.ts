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
  }
}, {})

/**
 * Build up the rpc calls for ApiPromise.create
 */
export const rpc = Object.entries(definitions).reduce((acc, [key, value]) => {
  return {
    ...acc,
    [key]: value.rpc,
  }
}, {})

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
  types
}
