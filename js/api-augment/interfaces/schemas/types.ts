// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Bytes, Struct, u16 } from '@polkadot/types-codec';

/** @name SchemaId */
export interface SchemaId extends u16 {}

/** @name SchemaModel */
export interface SchemaModel extends Bytes {}

/** @name SchemaResponse */
export interface SchemaResponse extends Struct {
  readonly schema_id: SchemaId;
  readonly model: SchemaModel;
}

export type PHANTOM_SCHEMAS = 'schemas';
