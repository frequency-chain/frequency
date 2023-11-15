use sp_std::vec::Vec;

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
	pub fn deserialize<'de, D: Deserializer<'de>>(
		deserializer: D,
	) -> Result<Option<Vec<u8>>, D::Error>
	where
		D: Deserializer<'de>,
	{
		impl_serde::serialize::deserialize(deserializer).map(|r| Some(r))
	}
}
/// Handle serializing and deserializing from `Vec<u8>` to a UTF-8 string
#[cfg(feature = "std")]
pub mod as_string {
	use super::*;
	use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};

	/// Serializes a `Vec<u8>` into a UTF-8 string
	pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
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

const PREFIX: &'static str = "<Bytes>";
const POSTFIX: &'static str = "</Bytes>";

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
	use parity_scale_codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};

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
}
