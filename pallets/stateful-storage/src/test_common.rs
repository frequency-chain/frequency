use common_primitives::{msa::MessageSourceId, schema::SchemaId};

///
/// Constants used both in benchmarks and tests
///
#[allow(unused)]
pub mod constants {
	use super::*;
	/// itemized schema id
	pub const ITEMIZED_SCHEMA: SchemaId = 100;
	/// paginated schema id
	pub const PAGINATED_SCHEMA: SchemaId = 101;
	/// Is used in benchmarks adn mocks to sign and verify a payload
	pub const BENCHMARK_SIGNATURE_ACCOUNT_SEED: &str =
		"replace rhythm attend tank sister accuse ancient piece tornado benefit rubber horror";
	/// Account mentioned above maps to the following msa id
	pub const SIGNATURE_MSA_ID: MessageSourceId = 105;

	/// additional unit test schemas

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
