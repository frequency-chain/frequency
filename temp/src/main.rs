use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::node::AccountId;
// use xcm_builder::location_conversion;
use hex;
use sp_core::{crypto::Ss58Codec, H256};
use sp_io::hashing::blake2_256;
use std::borrow::Borrow;
use xcm::{latest::prelude::*, v3::*};

fn main() {
	println!("version {:?}", VERSION);

	// bob conver
	// 5H56tPSET7HYxKTB4M3BpPLjdKr7eYBvJA1X9KSZhH1vK4Rd
	// alice
	// let location = MultiLocation {
	// 	parents: 1,
	// 	interior: X2(
	// 		Parachain(3000),
	// 		AccountId32 {
	// 			network: None,
	// 			id: [
	// 				142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135,
	// 				97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
	// 			],
	// 		},
	// 	),
	// };

	// enddy demo
	// let location = MultiLocation {
	// 	parents: 1,
	// 	interior: X2(
	// 		Parachain(3000),
	// 		AccountId32 {
	// 			network: None,
	// 			id: [72, 67, 94, 12, 137, 207, 124, 155, 131, 120, 70, 234, 246, 172, 159, 78, 58, 105, 135, 233, 99, 101, 14, 134, 216, 76, 216, 177, 240, 70, 150, 20],
	// 		},
	// 	),
	// };
	// gCTBPMUkVSd7jbcGn74wLG43zvxDYeLQHfdRuieQR5D1fVUZ1
	let location = MultiLocation {
		parents: 1,
		interior: X2(
			Parachain(3000),
			AccountId32 {
				network: None,
				id: [168, 218, 207, 163, 61, 156, 208, 97, 35, 130, 213, 54, 230, 187, 111, 76, 203, 62, 23, 103, 178, 238, 251, 219, 203, 158, 176, 24, 25, 218, 192, 13],
			},
		),
	};

	let encoded_tupple_hash = ("multiloc", location.borrow()).using_encoded(blake2_256);
	println!("endoded tupple hash {:?}", encoded_tupple_hash);
	let hex_address = hex::encode(encoded_tupple_hash.clone());
	println!("hex address {:?}", hex_address);

	let account_id = H256::from(encoded_tupple_hash);
	println!("accountId {:?}", account_id);

	let account: AccountId = encoded_tupple_hash.into();
	println!("accountId {:?}", account);

	// enddy demo convert
	// 5EVDg6MicQotBRNRH19dvvrYYeEezUeJdKP7WCgWfVEQj7NL

}

