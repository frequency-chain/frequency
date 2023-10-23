use crate::{
	tests::mock::*, types::AddKeyData, Config, Error, PayloadSignatureRegistryPointer,
	SignatureRegistryPointer,
};

use frame_support::{assert_noop, assert_ok};

use common_primitives::{node::BlockNumber, utils::wrap_binary_data};

use sp_core::{sr25519, Encode, Pair};
use sp_runtime::{BuildStorage, MultiSignature};

pub fn new_test_ext() -> sp_io::TestExternalities {
	set_max_signature_stored(20);
	set_max_public_keys_per_msa(10);

	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
pub fn cannot_register_too_many_signatures() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let mortality_block: BlockNumber = 3;

		let limit: u32 = <Test as Config>::MaxSignaturesStored::get().unwrap_or(0);
		for _i in 0..limit {
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, mortality_block.into()));
		}

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::SignatureRegistryLimitExceeded
		);
	})
}

#[test]
pub fn stores_signature_and_increments_count() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let mortality_block: BlockNumber = 51;
		let signature = generate_test_signature();
		assert_ok!(Msa::register_signature(&signature, mortality_block.into()));

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: signature.clone(),
				newest_expires_at: mortality_block.into(),
				oldest: signature.clone(),
				count: 1,
			}),
			<PayloadSignatureRegistryPointer<Test>>::get()
		);

		let oldest: MultiSignature = signature.clone();

		// Expect that the newest changes
		let signature_1 = generate_test_signature();
		assert_ok!(Msa::register_signature(&signature_1, mortality_block.into()));

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: signature_1.clone(),
				newest_expires_at: mortality_block.into(),
				oldest: signature.clone(),
				count: 2,
			}),
			<PayloadSignatureRegistryPointer<Test>>::get()
		);

		let mut newest: MultiSignature = signature_1.clone();

		// Fill up the registry
		let limit: u32 = <Test as Config>::MaxSignaturesStored::get().unwrap_or(0);
		for _i in 2..limit {
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, mortality_block.into()));
			newest = sig.clone();
		}

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: newest.clone(),
				newest_expires_at: mortality_block.into(),
				oldest: oldest.clone(),
				count: limit
			}),
			<PayloadSignatureRegistryPointer<Test>>::get()
		);

		run_to_block((mortality_block + 1).into());

		// Test that the next one changes the oldest signature.
		let signature_n = generate_test_signature();
		assert_ok!(Msa::register_signature(&signature_n, (mortality_block + 10).into()));

		assert_eq!(
			Some(SignatureRegistryPointer {
				newest: signature_n.clone(),
				newest_expires_at: (mortality_block + 10).into(),
				oldest: signature_1.clone(),
				count: limit,
			}),
			<PayloadSignatureRegistryPointer<Test>>::get()
		);
	})
}

#[test]
pub fn clears_stale_signatures_after_mortality_limit() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let mortality_block: BlockNumber = 3;

		let limit: u32 = <Test as Config>::MaxSignaturesStored::get().unwrap_or(0);
		for _i in 0..limit {
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, mortality_block.into()));
		}

		run_to_block((mortality_block).into());

		// Cannot do it yet as we are at the mortality_block

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, (mortality_block + 10).into()),
			Error::<Test>::SignatureRegistryLimitExceeded
		);

		run_to_block((mortality_block + 1).into());

		// Now it is OK as we are +1 past the mortality_block
		assert_ok!(Msa::register_signature(sig1, (mortality_block + 10).into()));
	})
}

#[test]
pub fn cannot_register_signature_with_mortality_out_of_bounds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mut mortality_block: BlockNumber = 11_323;

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofNotYetValid
		);

		mortality_block = 11_122;
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofHasExpired
		);
	})
}

struct TestCase {
	current: u32,
	mortality: u32,
	run_to: u32,
	expected_ok: bool,
}

#[test]
pub fn add_msa_key_replay_fails() {
	new_test_ext().execute_with(|| {
		// these should all fail replay
		let test_cases: Vec<TestCase> = vec![
			TestCase {
				current: 10_949u32,
				mortality: 11_001u32,
				run_to: 10_848u32,
				expected_ok: true,
			},
			TestCase { current: 1u32, mortality: 3u32, run_to: 5u32, expected_ok: false },
			TestCase { current: 99u32, mortality: 101u32, run_to: 100u32, expected_ok: true },
			TestCase {
				current: 1_100u32,
				mortality: 1_199u32,
				run_to: 1_198u32,
				expected_ok: true,
			},
			TestCase {
				current: 1_102u32,
				mortality: 1_201u32,
				run_to: 1_200u32,
				expected_ok: true,
			},
			TestCase {
				current: 1_099u32,
				mortality: 1_148u32,
				run_to: 1_101u32,
				expected_ok: true,
			},
			TestCase {
				current: 1_000_000u32,
				mortality: 1_000_000u32,
				run_to: 1_000_000u32,
				expected_ok: false,
			},
		];

		let (new_msa_id, key_pair_provider) = create_account();
		let account_provider = key_pair_provider.public();
		for tc in test_cases {
			System::set_block_number(tc.current);

			let (new_key_pair, _) = sr25519::Pair::generate();

			let add_new_key_data = AddKeyData {
				msa_id: new_msa_id,
				expiration: tc.mortality,
				new_public_key: new_key_pair.public().into(),
			};

			let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

			let signature_owner: MultiSignature =
				key_pair_provider.sign(&encode_data_new_key_data).into();

			let signature_new_key: MultiSignature =
				new_key_pair.sign(&encode_data_new_key_data).into();

			run_to_block(tc.run_to);

			let add_key_response: bool = Msa::add_public_key_to_msa(
				RuntimeOrigin::signed(account_provider.into()),
				account_provider.into(),
				signature_owner.clone(),
				signature_new_key,
				add_new_key_data.clone(),
			)
			.is_ok();

			assert_eq!(add_key_response, tc.expected_ok);
		}
	})
}
