use common_primitives::{msa::MessageSourceId, schema::SchemaId};

///
/// Constants used both in benchmarks and tests
///
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
}
