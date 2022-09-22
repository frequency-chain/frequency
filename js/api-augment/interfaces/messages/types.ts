// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { MessageSourceId } from '@frequency-chain/api-augment/msa';
import type { Bytes, Option, Struct, Vec, bool, u16, u32 } from '@polkadot/types-codec';
import type { BlockNumber } from '@polkadot/types/interfaces/runtime';

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
  readonly provider_msa_id: MessageSourceId;
  readonly msa_id: MessageSourceId;
  readonly index: u16;
  readonly block_number: BlockNumber;
  readonly payload_length: u32;
}

export type PHANTOM_MESSAGES = 'messages';
