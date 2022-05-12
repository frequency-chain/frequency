use sp_std::vec::Vec;

#[cfg(feature = "std")]
pub mod as_hex {
	use serde::{Deserializer, Serializer};

	pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		impl_serde::serialize::serialize(bytes.as_slice(), serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
		Ok(impl_serde::serialize::deserialize(deserializer)?)
	}
}

#[cfg(feature = "std")]
pub mod as_string {
	use super::*;
	use serde::{ser::Error, Deserialize, Deserializer, Serialize, Serializer};

	pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
		std::str::from_utf8(bytes)
			.map_err(|e| S::Error::custom(format!("Debug buffer contains invalid UTF8: {}", e)))?
			.serialize(serializer)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
		Ok(String::deserialize(deserializer)?.into_bytes())
	}
}

const PREFIX: &'static str = "<Bytes>";
const POSTFIX: &'static str = "</Bytes>";

pub fn wrap_binary_data(data: Vec<u8>) -> Vec<u8> {
	let mut encapsuled = PREFIX.as_bytes().to_vec();
	encapsuled.append(&mut data.clone());
	encapsuled.append(&mut POSTFIX.as_bytes().to_vec());
	encapsuled
}
