// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Bytes, Enum, Struct, u16 } from '@polkadot/types-codec';

/** @name ModelType */
export interface ModelType extends Enum {
  readonly isAvroBinary: boolean;
  readonly type: 'AvroBinary';
}

/** @name PayloadLocation */
export interface PayloadLocation extends Enum {
  readonly isOnChain: boolean;
  readonly isIpfs: boolean;
  readonly type: 'OnChain' | 'Ipfs';
}

/** @name SchemaId */
export interface SchemaId extends u16 {}

/** @name SchemaModel */
export interface SchemaModel extends Bytes {}

/** @name SchemaResponse */
export interface SchemaResponse extends Struct {
  readonly schema_id: SchemaId;
  readonly model: SchemaModel;
  readonly model_type: ModelType;
  readonly payload_location: PayloadLocation;
}

export type PHANTOM_SCHEMAS = 'schemas';
