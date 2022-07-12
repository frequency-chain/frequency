use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

pub mod base;
pub mod numeric;
pub mod string;
pub mod temporal;
pub mod types;
pub mod compression_codec;
use crate::parquet::types::ParquetType;
use crate::parquet::compression_codec::CompressionCodec;

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub struct ParquetModel {
  _type: ParquetType,
  compression: CompressionCodec,
  statistics: bool,
  bloom_filter: bool,
  optional: bool
}

