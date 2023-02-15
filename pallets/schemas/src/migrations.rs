//! Migrations for the schemas pallet.
//! Following migrations are required:
//! - Schema struct from v1 to v2

use super::*;
use crate::types::{Schema, SchemaV2};
use codec::{Decode, Encode};
use common_primitives::schema::SchemaSettings;
use frame_support::pallet_prelude::Weight;

fn migrate_schema_to_schema_v2<T: Config>() -> Weight {
	let mut weight: Weight = Weight::zero();
	weight
}
