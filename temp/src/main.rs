use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::node::AccountId;
// use xcm_builder::location_conversion;
use xcm::{latest::prelude::*, v3::*};
use sp_io::hashing::blake2_256;
use sp_core::H256;
use sp_core::crypto::Ss58Codec;
use std::borrow::Borrow;
use hex;


fn main() {
	println!("version {:?}", VERSION);

	let location = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(3000),
			AccountId32 {
				network: None,
				id: [
					142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135,
					97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
				],
			},
		),
	};

	let encoded_tupple_hash = ("multiloc", location.borrow()).using_encoded(blake2_256);
	println!("endoded tupple hash {:?}", encoded_tupple_hash);
	let hex_address = hex::encode(encoded_tupple_hash.clone());
	println!("hex address {:?}", hex_address);

	// let account_id = H256::from(encoded_tupple_hash);
	// println!("accountId {:?}", account_id);


	// let account: AccountId = encoded_tupple_hash.into();
	// println!("accountId {:?}", account);

	// 5H56tPSET7HYxKTB4M3BpPLjdKr7eYBvJA1X9KSZhH1vK4Rd
}
