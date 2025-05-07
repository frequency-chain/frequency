use crate::{
	stateful_child_tree::StatefulChildTree,
	test_common::{constants::*, test_utility::*},
	tests::mock::*,
	types::*,
	Config, Error,
};
use common_primitives::{
	node::EIP712Encode,
	signatures::{UnifiedSignature, UnifiedSigner},
	utils::wrap_binary_data,
};
use frame_support::{assert_err, assert_ok};
use parity_scale_codec::Encode;
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::{bytes::from_hex, ecdsa, Pair};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};

#[test]
fn is_empty_false_for_non_empty_page() {
	let page: ItemizedPage<Test> =
		create_itemized_page_from::<Test>(None, &[generate_payload_bytes(None)]);

	assert_eq!(page.is_empty(), false);
}

#[test]
fn is_empty_true_for_empty_page() {
	let page: ItemizedPage<Test> = create_itemized_page_from::<Test>(None, &[]);

	assert_eq!(page.is_empty(), true);
}

#[test]
fn signature_v2_replay_on_existing_page_errors() {
	new_test_ext().execute_with(|| {
		// Setup
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let keys = (schema_id, page_id);
		let page_a: PaginatedPage<Test> = generate_page(Some(1), Some(1));
		let page_b: PaginatedPage<Test> = generate_page(Some(2), Some(2));
		let payload_a_to_b = PaginatedUpsertSignaturePayloadV2 {
			expiration: 10,
			schema_id,
			page_id,
			target_hash: page_a.get_hash(),
			payload: page_b.data.clone(),
		};

		// Set up initial state A
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&page_a,
		);

		// Make sure we successfully apply state transition A -> B
		let encoded_payload = wrap_binary_data(payload_a_to_b.encode());
		let owner_a_to_b_signature: MultiSignature = pair.sign(&encoded_payload).into();
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature_v2(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_a_to_b_signature.clone(),
			payload_a_to_b.clone()
		));

		// Read back page state & get hash
		let current_page: PaginatedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap()
			.expect("no page read");

		// Make sure we successfully apply state transition B -> A
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			current_page.get_hash(),
			page_a.data
		));

		// Signature replay A -> B should fail
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_a_to_b_signature,
				payload_a_to_b
			),
			Error::<Test>::StalePageState
		);
	})
}

// NOTE: This is a known issue. When it's fixed, this test will start failing & we can change the test assertion.
#[test]
fn signature_v2_replay_on_deleted_page_check() {
	new_test_ext().execute_with(|| {
		// Setup
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let keys = (schema_id, page_id);
		let page_a: PaginatedPage<Test> = generate_page(Some(1), Some(1));
		let payload_null_to_a = PaginatedUpsertSignaturePayloadV2 {
			expiration: 10,
			schema_id,
			page_id,
			target_hash: NONEXISTENT_PAGE_HASH,
			payload: page_a.data.clone(),
		};

		// Make sure we successfully apply state transition Null -> A
		let encoded_payload = wrap_binary_data(payload_null_to_a.encode());
		let owner_null_to_a_signature: MultiSignature = pair.sign(&encoded_payload).into();
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature_v2(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_null_to_a_signature.clone(),
			payload_null_to_a.clone()
		));

		// Read back page state & get hash
		let current_page: PaginatedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap()
			.expect("no page read");

		// Make sure we successfully delete the page
		assert_ok!(StatefulStoragePallet::delete_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			current_page.get_hash(),
		));

		// Signature replay A -> B (change assertion when this vulnerability is fixed)
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature_v2(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_null_to_a_signature,
			payload_null_to_a
		));
	})
}

#[test]
fn ethereum_eip712_signatures_for_paginated_delete_signature_should_work() {
	new_test_ext().execute_with(|| {
		let payload: PaginatedDeleteSignaturePayloadV2<Test> = PaginatedDeleteSignaturePayloadV2 {
			schema_id: 10,
			page_id: 5,
			target_hash: 1982672367,
			expiration: 100,
		};
		let encoded_payload = payload.encode_eip_712();

		let signature_raw = from_hex("0xdc7212abf872317936e9d499705ebf6a3464891fd0798c50025d563341b36b844dda940fe511fcc59769a06ed67df84b19c71c61888c1160461a2be811754ec71c").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}

#[test]
fn ethereum_eip712_signatures_for_paginated_upsert_signature_should_work() {
	new_test_ext().execute_with(|| {
		let payload: PaginatedUpsertSignaturePayloadV2<Test> = PaginatedUpsertSignaturePayloadV2 {
			schema_id: 10,
			page_id: 5,
			target_hash: 1982672367,
			expiration: 100,
			payload: from_hex("0x0000f20f0efece05af6bee4fe6b24db6febe6988d24f62015e23d1f1ce9b7040301a8a3cefc00c1162067f70d004ed86588cd6183b3564b9d92e9924cba2244a9e9f85e334e1b8bb5e8f770a81866c4c406cfadaef2a8f93bf8b89a5f4efce86a3cfc1ff4d93a8d362d630bf16ac5766de75c5fef57af69b147cb629ad8c1359691c9ab6f5380527e3c18d71b0480747a9628693b86473f1e7f01abe5bb7eadb76fad361669241e74bffed13b49e5c7dd08a8d3a7722bcbfad494627256c7e77f2c9b2b596a5f468a82e06f5905d13707355e2dd77d9b9480ec959a7f870b68fce122ddad81e06fe4dec48e01206df9e6e3a4e0d928b722220ec199d22324fb2ce6a8b6581175476d940bd499435091c0b0b3d0303e3bf60732ff8e898223a39aa8623cf33a1623af2cfc8d6fc83f4b53ed32ef4fdab29035150d9dc3b7e60ac91fb1fbb6895a007913fee3940c8f8e3ececcf50b3076b65c62f8bd8c039f90182d89bf4c0a2f426e957a8732a9936f8a0aab8b3c18183eb605f5a0bac859eb7d6d88eea4c02444116a6dec0a790d83abce10de20bb29c3f980864eb23f555422ff36421e14df39bb53606cbb4dda195c321ce4fa4fc7143ac267ae811b6949d51f521ddaf08a3663dd3a1eb2b783b31dcbfc830f6f542499857767c39fb85a29ebf175e0f0877e89a9e564307a7d7eb9d1f8401f0a65382cfc051fa5d34829381f624e2a556e2c3002eea63e50ad8463bf8ac6096249983f8d1925c0d9392e67203be98daae7305b52962b95fbeca76c7db2ceec6208d21efb68350aedec3a48cba0c9112c93efb98d363dfa26471b163b05d1655f0af7d867fd25dcc4d5e1a5cb3934586ba7a418f489439f3f551c1017bde009dba49dbc132c7066eecea25e2e231491f136a1fe9a83f0f1091c7e9b2cf8d24541fe21af50a46e6537f80ed2065da8842b928cb27ec23169c129863d5540f05a4dac070f3c834cfe998503f4a42a7ae4e2b1ef694e7600027e48721560d8d66a46c5ac7899d71985c7ccb60d4225cd08307c497138b6cbb00803cf9be1251798c6ddd8c972f78455a43acc7bf6278dda25946dc8638c20042ba333ee6e6da605bb05b1d01c5715c1723a2d6147eca637f6a7f50b807476e482c29769e1b94b43ac7b1921e99c60c795cf034a706c28befbcbe448f926ca212eb4607157dd9dd30c89b9a885202789b2ac9a6e030c1d87d5341c59b64105a88caa13cdcc42bbdda752169234e39d94f6aadcfdd99e80e9eb3a10ea5a80ba4825370ad9935d5b8cda568d2c7db63a8e016cb78bd0d657f5d1f6916fb48a3678d973dc8835a49f5e2a0ff07698c360492f568def07397129d290a86cefd0524c826b73e85c48a8e525aafe77bd0c1e9a0c2a4a4ea96343adfe81fad9ef12200ed90c6c476906d710e16d3af77a4e18164").unwrap().try_into().unwrap()
		};
		let encoded_payload = payload.encode_eip_712();

		let signature_raw = from_hex("0xea418adf5fa93b7dd799cc4bb72923403f73191f952cca6515f96fc6f49aebea1f31db30106ae32a30d69ee89e418a3400a9111670e770a7abcb13b6358c237e1c").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}

#[test]
fn ethereum_eip712_signatures_for_itemized_signature_should_work() {
	new_test_ext().execute_with(|| {
		let payload: ItemizedSignaturePayloadV2<Test> = ItemizedSignaturePayloadV2 {
			schema_id: 10,
			target_hash: 1982672367,
			expiration: 100,
			actions: vec![
				ItemAction::Add {
					data: from_hex("0x40a6836ea489047852d3f0297f8fe8ad6779793af4e9c6274c230c207b9b825026").unwrap().try_into().unwrap()
				},
				ItemAction::Delete {
					index: 2
				}
			].try_into().unwrap()
		};
		let encoded_payload = payload.encode_eip_712();

		let signature_raw = from_hex("0x7efb5407412c745f40713ba0922e228bf5f2b628423817a7a333a36902df0df45ef4cfd3c309fcaf9c0fc72e91a96b5456740b345283fc4525f5b09802ad1c0d1c").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}
