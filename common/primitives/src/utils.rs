extern crate alloc;
use alloc::vec::Vec;

/// Mainnet Genesis Hash 0x4a587bf17a404e3572747add7aab7bbe56e805a5479c6c436f07f36fcc8d3ae1
pub const MAINNET_GENESIS_HASH: &[u8] = &[
	74u8, 88, 123, 241, 122, 64, 78, 53, 114, 116, 122, 221, 122, 171, 123, 190, 86, 232, 5, 165,
	71, 156, 108, 67, 111, 7, 243, 111, 204, 141, 58, 225,
];

/// Frequency Testnet on Paseo Genesis Hash 0x203c6838fc78ea3660a2f298a58d859519c72a5efdc0f194abd6f0d5ce1838e0
pub const TESTNET_ON_PASEO_GENESIS_HASH: &[u8] = &[
	32, 60, 104, 56, 252, 120, 234, 54, 96, 162, 242, 152, 165, 141, 133, 149, 25, 199, 42, 94,
	253, 192, 241, 148, 171, 214, 240, 213, 206, 24, 56, 224,
];

/// An enum that shows the detected chain type
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedChainType {
	/// An unknown chain, it can be a local or development chain
	Unknown,
	/// Frequency Mainnet
	FrequencyMainNet,
	/// Frequency Paseo Testnet
	FrequencyPaseoTestNet,
}

/// Finds the chain type by genesis hash
pub fn get_chain_type_by_genesis_hash(genesis_hash: &[u8]) -> DetectedChainType {
	match genesis_hash {
		MAINNET_GENESIS_HASH => DetectedChainType::FrequencyMainNet,
		TESTNET_ON_PASEO_GENESIS_HASH => DetectedChainType::FrequencyPaseoTestNet,
		_ => DetectedChainType::Unknown,
	}
}

/// Generic function for converting any unsigned integer to a 32-byte array compatible with ETH abi
pub fn to_abi_compatible_number<T: Into<u128>>(value: T) -> [u8; 32] {
	let value_u128: u128 = value.into();
	// Convert to big-endian bytes
	let bytes = value_u128.to_be_bytes();
	// Copy the non-zero part to the end of the result array
	let start_idx = 32 - bytes.len();
	let mut result = [0u8; 32];
	result[start_idx..].copy_from_slice(&bytes);
	result
}

/// Handle serializing and deserializing from `Vec<u8>` to hexadecimal
#[cfg(feature = "std")]
pub mod as_hex {
	use serde::{Deserializer, Serializer};

	/// Serializes a `Vec<u8>` into a hexadecimal string
	pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		impl_serde::serialize::serialize(bytes.as_slice(), serializer)
	}

	/// Deserializes a hexadecimal string into a `Vec<u8>`
	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
		impl_serde::serialize::deserialize(deserializer)
	}
}

/// Handle serializing and deserializing from `Option<Vec<u8>>` to hexadecimal
#[cfg(feature = "std")]
pub mod as_hex_option {
	use serde::{Deserializer, Serializer};

	/// Serializes a `Vec<u8>` into a hexadecimal string
	pub fn serialize<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match bytes {
			Some(bytes) => impl_serde::serialize::serialize(bytes.as_slice(), serializer),
			None => serializer.serialize_none(),
		}
	}

	/// Deserializes a hexadecimal string into a `Vec<u8>`
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
	where
		D: Deserializer<'de>,
	{
		impl_serde::serialize::deserialize(deserializer).map(Some)
	}
}
/// Handle serializing and deserializing from `Vec<u8>` to a UTF-8 string
#[cfg(feature = "std")]
pub mod as_string {
	use super::*;
	use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};

	/// Serializes a `Vec<u8>` into a UTF-8 string
	pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
		std::str::from_utf8(bytes)
			.map_err(|e| S::Error::custom(format!("Debug buffer contains invalid UTF8: {}", e)))?
			.serialize(serializer)
	}

	/// Serializes a UTF-8 string into a `Vec<u8>`
	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
		Ok(String::deserialize(deserializer)?.into_bytes())
	}
}

/// Handle serializing and deserializing from `Option<Vec<u8>>` to a UTF-8 string
#[cfg(feature = "std")]
pub mod as_string_option {
	use super::*;
	use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};

	/// Serializes a `Option<Vec<u8>>` into a UTF-8 string if Ok()
	pub fn serialize<S: Serializer>(
		bytes: &Option<Vec<u8>>,
		serializer: S,
	) -> Result<S::Ok, S::Error> {
		match bytes {
			Some(bytes) => std::str::from_utf8(bytes)
				.map_err(|e| {
					S::Error::custom(format!("Debug buffer contains invalid UTF8: {}", e))
				})?
				.serialize(serializer),
			None => serializer.serialize_none(),
		}
	}

	/// Deserializes a UTF-8 string into a `Option<Vec<u8>>`
	pub fn deserialize<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Option<Vec<u8>>, D::Error> {
		let bytes = String::deserialize(deserializer)?.into_bytes();
		Ok(match bytes.len() {
			0 => None,
			_ => Some(bytes),
		})
	}
}

const PREFIX: &str = "<Bytes>";
const POSTFIX: &str = "</Bytes>";

/// Wraps `PREFIX` and `POSTFIX` around a `Vec<u8>`
/// Returns `PREFIX` ++ `data` ++ `POSTFIX`
pub fn wrap_binary_data(data: Vec<u8>) -> Vec<u8> {
	let mut encapsuled = PREFIX.as_bytes().to_vec();
	encapsuled.append(&mut data.clone());
	encapsuled.append(&mut POSTFIX.as_bytes().to_vec());
	encapsuled
}

#[cfg(test)]
mod tests {
	use super::*;
	use impl_serde::serialize::from_hex;
	use parity_scale_codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};
	use sp_core::U256;

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
	struct TestAsHex {
		#[cfg_attr(feature = "std", serde(with = "as_hex",))]
		pub data: Vec<u8>,
	}

	#[test]
	fn as_hex_can_serialize() {
		let test_data = TestAsHex { data: vec![1, 2, 3, 4] };
		let result = serde_json::to_string(&test_data);
		assert!(result.is_ok());
		assert_eq!("{\"data\":\"0x01020304\"}", result.unwrap());
	}

	#[test]
	fn as_hex_can_deserialize() {
		let result: Result<TestAsHex, serde_json::Error> =
			serde_json::from_str("{\"data\":\"0x01020304\"}");
		assert!(result.is_ok());
		assert_eq!(vec![1, 2, 3, 4], result.unwrap().data);
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
	struct TestAsHexOption {
		#[cfg_attr(
			feature = "std",
			serde(with = "as_hex_option", skip_serializing_if = "Option::is_none", default)
		)]
		pub data: Option<Vec<u8>>,
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
	struct TestAsHexOptionNull {
		#[cfg_attr(feature = "std", serde(with = "as_hex_option", default))]
		pub data: Option<Vec<u8>>,
	}

	#[test]
	fn as_hex_option_can_serialize() {
		let test_data = TestAsHexOption { data: Some(vec![1, 2, 3, 4]) };
		let result = serde_json::to_string(&test_data);
		assert!(result.is_ok());
		assert_eq!("{\"data\":\"0x01020304\"}", result.unwrap());
	}

	#[test]
	fn as_hex_option_can_deserialize() {
		let result: Result<TestAsHexOption, serde_json::Error> =
			serde_json::from_str("{\"data\":\"0x01020304\"}");
		assert!(result.is_ok());
		assert_eq!(Some(vec![1, 2, 3, 4]), result.unwrap().data);
	}

	#[test]
	fn as_hex_option_can_serialize_nothing_with_skip() {
		let test_data = TestAsHexOption { data: None };
		let result = serde_json::to_string(&test_data);
		assert!(result.is_ok());
		assert_eq!("{}", result.unwrap());
	}

	#[test]
	fn as_hex_option_can_serialize_nothing_as_null() {
		let test_data = TestAsHexOptionNull { data: None };
		let result = serde_json::to_string(&test_data);
		assert!(result.is_ok());
		assert_eq!("{\"data\":null}", result.unwrap());
	}

	#[test]
	fn as_hex_option_can_deserialize_nothing() {
		let result: Result<TestAsHexOption, serde_json::Error> = serde_json::from_str("{}");
		assert!(result.is_ok());
		assert_eq!(None, result.unwrap().data);
	}

	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
	struct TestAsString {
		#[cfg_attr(feature = "std", serde(with = "as_string"))]
		pub data: Vec<u8>,
	}

	#[test]
	fn as_string_can_serialize() {
		let test_data = TestAsString {
			data: vec![
				0xe8, 0x95, 0x99, 0x49, 0xdd, 0x9d, 0xcd, 0x99, 0xe0, 0xbc, 0x8d, 0x4c, 0xd0, 0xbc,
			],
		};
		let result = serde_json::to_string(&test_data);
		assert!(result.is_ok());
		assert_eq!("{\"data\":\"蕙Iݝ͙།Lм\"}", result.unwrap());
	}

	#[test]
	fn as_string_can_deserialize() {
		let result: Result<TestAsString, serde_json::Error> =
			serde_json::from_str("{\"data\":\"蕙Iݝ͙།Lм\"}");
		assert!(result.is_ok());
		assert_eq!(
			vec![
				0xe8, 0x95, 0x99, 0x49, 0xdd, 0x9d, 0xcd, 0x99, 0xe0, 0xbc, 0x8d, 0x4c, 0xd0, 0xbc
			],
			result.unwrap().data
		);
	}

	#[test]
	fn as_string_errors_for_bad_utf8_vec() {
		let test_data = TestAsString { data: vec![0xc3, 0x28] };
		let result = serde_json::to_string(&test_data);
		assert!(result.is_err());
	}

	#[test]
	fn as_string_errors_for_bad_utf8_str() {
		let result: Result<TestAsString, serde_json::Error> =
			serde_json::from_str("{\"data\":\"\\xa0\\xa1\"}");
		assert!(result.is_err());
	}

	#[test]
	fn get_chain_type_by_genesis_hash_with_mainnet_genesis_should_get_mainnet() {
		// arrange
		let known_genesis =
			from_hex("4a587bf17a404e3572747add7aab7bbe56e805a5479c6c436f07f36fcc8d3ae1").unwrap();

		// act
		let detected = get_chain_type_by_genesis_hash(&known_genesis);

		// assert
		assert_eq!(detected, DetectedChainType::FrequencyMainNet);
	}

	#[test]
	fn get_chain_type_by_genesis_hash_with_paseo_genesis_should_get_paseo() {
		// arrange
		let known_genesis =
			from_hex("203c6838fc78ea3660a2f298a58d859519c72a5efdc0f194abd6f0d5ce1838e0").unwrap();

		// act
		let detected = get_chain_type_by_genesis_hash(&known_genesis);

		// assert
		assert_eq!(detected, DetectedChainType::FrequencyPaseoTestNet);
	}

	#[test]
	fn abi_compatible_number_should_work_with_different_types() {
		// For u8
		let u8_val: u8 = 42;
		let coded_u8_val = to_abi_compatible_number(u8_val);
		let u8_val: U256 = u8_val.into();
		assert_eq!(
			coded_u8_val.to_vec(),
			sp_core::bytes::from_hex(&format!("0x{:064x}", u8_val)).unwrap()
		);

		// For u16
		let u16_val: u16 = 12345;
		let coded_u16_val = to_abi_compatible_number(u16_val);
		let u16_val: U256 = u16_val.into();
		assert_eq!(
			coded_u16_val.to_vec(),
			sp_core::bytes::from_hex(&format!("0x{:064x}", u16_val)).unwrap()
		);

		// For u32
		let u32_val: u32 = 305419896;
		let coded_u32_val = to_abi_compatible_number(u32_val);
		let u32_val: U256 = u32_val.into();
		assert_eq!(
			coded_u32_val.to_vec(),
			sp_core::bytes::from_hex(&format!("0x{:064x}", u32_val)).unwrap()
		);

		// For u64
		let u64_val: u64 = 1234567890123456789;
		let coded_u64_val = to_abi_compatible_number(u64_val);
		let u64_val: U256 = u64_val.into();
		assert_eq!(
			coded_u64_val.to_vec(),
			sp_core::bytes::from_hex(&format!("0x{:064x}", u64_val)).unwrap()
		);

		// For u128
		let u128_val: u128 = 340282366920938463463374607431768211455; // Max u128 value
		let coded_u128_val = to_abi_compatible_number(u128_val);
		let u128_val: U256 = u128_val.into();
		assert_eq!(
			coded_u128_val.to_vec(),
			sp_core::bytes::from_hex(&format!("0x{:064x}", u128_val)).unwrap()
		);
	}
}
