import "./interfaces/types-lookup";
import "./interfaces/augment-api";
import "./interfaces/augment-types";
import * as definitions from "./interfaces/definitions";

export const types = Object.entries(definitions).reduce((acc, [_key, value]) => {
  return {
    ...acc,
    ...value.types,
  }
}, {})

export const rpc = Object.entries(definitions).reduce((acc, [key, value]) => {
  return {
    ...acc,
    [key]: value.rpc,
  }
}, {})

export const options = {
  rpc,
  types
}
