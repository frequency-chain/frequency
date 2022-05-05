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
