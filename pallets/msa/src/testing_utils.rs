use sp_core::{sr25519, Pair};

use crate::Config;

use common_primitives::schema::SchemaValidator;

pub fn create_two_keypairs() -> (sr25519::Pair, sr25519::Pair) {
	// fn create_two_keypairs() -> (Public, Public) {
	let (pair1, _) = sr25519::Pair::generate();
	let (pair2, _) = sr25519::Pair::generate();
	(pair1, pair2)
	// (pair1.public(), pair2.public())
}

pub fn set_schema_count<T: Config>(n: u16) {
	<T>::SchemaValidator::set_schema_count(n);
}
