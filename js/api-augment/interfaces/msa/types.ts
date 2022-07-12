// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Struct, u32, u64 } from '@polkadot/types-codec';
import type { AccountId, BlockNumber } from '@polkadot/types/interfaces/runtime';

/** @name KeyInfoResponse */
export interface KeyInfoResponse extends Struct {
  readonly key: AccountId;
  readonly msaId: MessageSourceId;
  readonly nonce: u32;
  readonly expired: BlockNumber;
}

/** @name MessageSourceId */
export interface MessageSourceId extends u64 {}

export type PHANTOM_MSA = 'msa';
