use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ParquetModel {
  _type: ParquetType,
  compression: CompressionCodec,
  statistics: bool,
  bloom_filter: bool,
  optional: bool
}

pub trait Model {
  fn get_serialized_data(&self) -> Vec<u8>;
}

impl Model for ParquetModel {
  fn get_serialized_data(&self) -> Vec<u8> {
    // TBD serialization
    todo!()
  }
}
