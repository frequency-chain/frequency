use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{ensure_dot_asset_exists_on_frequency, mint_dot_on_frequency},
};

use codec::Encode;

use emulated_integration_tests_common::impls::ParaId;

fn assert_open_channel_request_received(_t: FrequencyToRelayTest) {
	Westend::assert_ump_queue_processed(true, None, None);
}

fn frequency_to_relay_send_xcm(t: FrequencyToRelayTest) -> DispatchResult {
	let assets = t.args.assets;
	let xcm_fee: Asset = (Location::here(), 1_000_000_000_000u128).into();

	type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;

	type WestendRuntimeCall = <Westend as Chain>::RuntimeCall;

	let xcm = Xcm::<()>(vec![
		WithdrawAsset(assets),
		BuyExecution { fees: xcm_fee, weight_limit: Unlimited },
		Transact {
			origin_kind: OriginKind::Native,
			fallback_max_weight: None,
			call: WestendRuntimeCall::Hrmp(
				polkadot_runtime_parachains::hrmp::Call::establish_channel_with_system {
					target_system_chain: ParaId::new(1000u32),
				},
			)
			.encode()
			.into(),
		},
		DepositAsset {
			assets: Wild(All),
			beneficiary: AccountId32Junction {
				network: None,
				id: FrequencyWestendSender::get().into(),
			}
			.into(),
		},
	]);

	let inner_call: <FrequencyWestend as Chain>::RuntimeCall =
		RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
			dest: bx!(VersionedLocation::from(Location::parent())),
			message: bx!(VersionedXcm::from(xcm)),
		});

	let _ =
		<FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(t.root_origin, bx!(inner_call));

	Ok(())
}

pub fn setup_parent_asset_on_frequency(
	account: AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	amount: Balance,
) {
	ensure_dot_asset_exists_on_frequency();
	mint_dot_on_frequency(account, amount)
}

pub fn fund_sov_frequency_on_westend(amount: Balance) {
	let freq_location = Westend::child_location_of(FrequencyWestend::para_id());
	let sov_account = Westend::sovereign_account_id_of(freq_location);
	Westend::fund_accounts(vec![(sov_account.into(), amount)]);
}

// =========================================================================
// ========= Open Channel - Frequency<>Relay ===========
// =========================================================================
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::send_open_channel_request -- --nocapture
/// Tests reserve transfer of DOT from Frequency to Relay (Westend).
/// Ensures sovereign account on relay is debited and receiver is credited.
#[test]
fn send_open_channel_request() {
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	let amount_to_send: Balance = WESTEND_ED * 1000;

	setup_parent_asset_on_frequency(sender.clone(), amount_to_send * 2);
	fund_sov_frequency_on_westend(amount_to_send * 4);

	let assets: Assets = (Location::here(), amount_to_send * 2).into();
	let destination = FrequencyWestend::parent_location();

	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver,
			amount_to_send,
			assets.clone(),
			None,
			0,
		),
	};

	let mut test = FrequencyToRelayTest::new(test_args);

	let dot_location = WestendLocation::get();

	let treasury_account_balance = foreign_balance_on!(
		FrequencyWestend,
		dot_location.clone(),
		&FrequencyTreasuryAccount::get()
	);
	assert_eq!(treasury_account_balance, 0u128);

	test.set_assertion::<Westend>(assert_open_channel_request_received);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_relay_send_xcm);
	test.assert();

	let treasury_account_balance = foreign_balance_on!(
		FrequencyWestend,
		dot_location.clone(),
		&FrequencyTreasuryAccount::get()
	);
	assert!(treasury_account_balance == 0u128, "Treasury account should NOT have been credited");
}
