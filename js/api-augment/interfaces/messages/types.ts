// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { MessageSourceId } from '@dsnp/frequency-api-augment/msa';
import type { Bytes, Option, Struct, Vec, bool, u16, u32 } from '@polkadot/types-codec';
import type { AccountId, BlockNumber } from '@polkadot/types/interfaces/runtime';

/** @name BlockPaginationRequest */
export interface BlockPaginationRequest extends Struct {
  readonly from_block: BlockNumber;
  readonly from_index: u32;
  readonly to_block: BlockNumber;
  readonly page_size: u32;
}

/** @name BlockPaginationResponseMessage */
export interface BlockPaginationResponseMessage extends Struct {
  readonly content: Vec<MessageResponse>;
  readonly has_next: bool;
  readonly next_block: Option<BlockNumber>;
  readonly next_index: Option<u32>;
}

/** @name MessageResponse */
export interface MessageResponse extends Struct {
  readonly payload: Bytes;
  readonly provider_key: AccountId;
  readonly msa_id: MessageSourceId;
  readonly index: u16;
  readonly block_number: BlockNumber;
}

export type PHANTOM_MESSAGES = 'messages';
