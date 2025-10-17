use common_primitives::{msa::MessageSourceId, schema::SchemaId};

/// Constants used for tests only
#[cfg(test)]
mod test_constants {
	use common_primitives::msa::SchemaId;

	/// Itemized
	pub const UNDELEGATED_ITEMIZED_APPEND_ONLY_SCHEMA: SchemaId = 102;
	pub const ITEMIZED_APPEND_ONLY_SCHEMA: SchemaId = 103;
	pub const ITEMIZED_SIGNATURE_REQUIRED_SCHEMA: SchemaId = 104;
	pub const UNDELEGATED_ITEMIZED_SCHEMA: SchemaId = 105;
	/// Paginated
	pub const PAGINATED_SIGNED_SCHEMA: SchemaId = 106;
	pub const PAGINATED_APPEND_ONLY_SCHEMA: SchemaId = 107;
	pub const UNDELEGATED_PAGINATED_SCHEMA: SchemaId = 108;
}

///
/// Constants used both in benchmarks and tests
///
#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod constants {
	use super::*;
	/// itemized schema id
	pub const ITEMIZED_SCHEMA: SchemaId = 100;
	/// paginated schema id
	pub const PAGINATED_SCHEMA: SchemaId = 101;
	/// Is used in benchmarks and mocks to sign and verify a payload
	pub const BENCHMARK_SIGNATURE_ACCOUNT_SEED: &str =
		"replace rhythm attend tank sister accuse ancient piece tornado benefit rubber horror";
	/// Account mentioned above maps to the following msa id
	pub const SIGNATURE_MSA_ID: MessageSourceId = 105;
	// additional unit test schemas
	#[cfg(test)]
	pub use test_constants::*;
}

#[cfg(test)]
pub mod test_utility {
	use crate::{
		pallet, tests::mock::Test, types::PageVersion, Config, ItemHeader, ItemizedPage, Page,
	};
	use common_primitives::{
		schema::{ModelType, PayloadLocation, SchemaId},
		stateful_storage::PageNonce,
	};
	use frame_support::BoundedVec;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_core::Get;

	pub type ItemizedPageSize = <Test as Config>::MaxItemizedPageSizeBytes;
	pub type PaginatedPageSize = <Test as Config>::MaxPaginatedPageSizeBytes;
	pub type ItemizedBlobSize = <Test as Config>::MaxItemizedBlobSizeBytes;

	pub const NONEXISTENT_PAGE_HASH: u32 = 0;

	pub fn generate_payload_bytes<T: Get<u32>>(id: Option<u8>) -> BoundedVec<u8, T> {
		let value = id.unwrap_or(1);
		format!("{{'type':{value}, 'description':'another test description {value}'}}")
			.as_bytes()
			.to_vec()
			.try_into()
			.unwrap()
	}

	pub fn generate_page<T: Get<u32>>(
		schema_id: Option<SchemaId>,
		in_nonce: Option<PageNonce>,
		id: Option<u8>,
	) -> Page<T> {
		let nonce = in_nonce.unwrap_or_default();
		Page::<T> {
			page_version: PageVersion::default(),
			schema_id,
			nonce,
			data: generate_payload_bytes(id),
		}
	}

	pub fn add_itemized_payload_to_buffer<T: Config>(buffer: &mut Vec<u8>, payload: &[u8]) {
		buffer.extend_from_slice(
			&ItemHeader {
				item_version: Default::default(),
				schema_id: 0,
				payload_len: payload.len() as u16,
			}
			.encode()[..],
		);
		buffer.extend_from_slice(payload);
	}

	pub fn create_itemized_page_from<T: pallet::Config>(
		nonce_in: Option<PageNonce>,
		payloads: &[BoundedVec<u8, ItemizedBlobSize>],
	) -> ItemizedPage<T> {
		let nonce = nonce_in.unwrap_or_default();
		let mut buffer: Vec<u8> = vec![];
		for p in payloads {
			add_itemized_payload_to_buffer::<T>(&mut buffer, p.as_slice());
		}
		let data = BoundedVec::<u8, T::MaxItemizedPageSizeBytes>::try_from(buffer).unwrap();
		ItemizedPage::<T> { page_version: Default::default(), schema_id: None, nonce, data }
	}
	#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
	/// A structure defining a Schema
	pub struct TestStruct {
		pub model_type: ModelType,
		pub payload_location: PayloadLocation,
		pub number: u64,
	}
}
