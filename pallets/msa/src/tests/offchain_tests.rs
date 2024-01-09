use frame_support::{
	assert_err, assert_noop, assert_ok,
	dispatch::{GetDispatchInfo, Pays},
};
use frame_support::pallet_prelude::Hooks;

use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};

use crate::{ensure, tests::mock::*, Config, DispatchResult, Error, Event, ProviderToRegistryEntry, Pallet, RPC_FINALIZED_BLOCK_REQUEST_URL, RPC_FINALIZED_BLOCK_REQUEST_BODY, FinalizedBlockResponse, HTTP_REQUEST_DEADLINE_MILLISECONDS};

use common_primitives::{
	msa::{
		Delegation, DelegatorId, ProviderId, ProviderRegistryEntry, SchemaGrant,
		SchemaGrantValidator,
	},
	node::BlockNumber,
	schema::{SchemaId, SchemaValidator},
	utils::wrap_binary_data,
};
use pretty_assertions::assert_eq;
use sp_core::crypto::AccountId32;
use sp_core::offchain::Duration;
use sp_runtime::traits::One;

fn fill_accounts<T: Config>(accounts: usize) {
	for mut n in 1..=accounts {
		let mut raw = [0u8; 32];
		let mut c = 0usize;
		while n > 0 {
			raw[c] = (n % 10) as u8;
			c += 1;
			n /= 10;
		}
		let new_key = AccountId32::new(raw);
		let public_key =
			T::AccountId::decode(&mut &new_key.encode()[..]).expect("should decode");
		let _ = Pallet::<T>::create_account(public_key, |_| -> DispatchResult { Ok(()) });
	}
}

#[test]
pub fn should_fill_the_db() {
	new_test_with_offchain_ext().execute_with(|| {
		fill_accounts::<Test>(10);
		let block_number = BlockNumberFor::<Test>::one();
		let response = FinalizedBlockResponse {
			result: "0x5685c63b9df72b59f6fa8e1223532c041d15a1abbe39a6d2a48d6565a091839b".to_string()
		};
		let deadline =
			sp_io::offchain::timestamp().add(Duration::from_millis(HTTP_REQUEST_DEADLINE_MILLISECONDS));
		let id = sp_io::offchain::http_request_start(
			"POST",
			RPC_FINALIZED_BLOCK_REQUEST_URL,
			&[],
		).ok().unwrap();
		sp_io::offchain::http_request_add_header(id, "Content-Type", "application/json").ok().unwrap();
		sp_io::offchain::http_request_write_body(id, RPC_FINALIZED_BLOCK_REQUEST_BODY, Some(deadline)).ok().unwrap();

		// TODO: fix this

		Msa::on_initialize(block_number);
		Msa::offchain_worker(block_number);

	});
}

