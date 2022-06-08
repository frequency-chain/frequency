import * as definitions from "./definitions/index.js";

// load all custom types and RPCs from definitions
export const types = Object.values(definitions).reduce(
  (res, { types }) => ({ ...res, ...types }),
  {}
);

export const rpc = Object.values(definitions).reduce(
  (res, { rpc }) => ({ ...res, ...rpc }),
  {}
);
