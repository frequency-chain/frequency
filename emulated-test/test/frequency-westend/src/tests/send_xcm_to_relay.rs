use crate::{
	foreign_balance_on,
	imports::*,
	tests::utils::{
		ensure_dot_asset_exists_on_frequency, mint_dot_on_frequency, mint_dot_on_frequency_v2,
	},
};

use emulated_integration_tests_common::xcm_emulator::ConvertLocation;

fn frequency_to_relay_send_xcm(t: FrequencyToRelayTest) -> DispatchResult {
	let assets = t.args.assets;

	type RuntimeCall = <FrequencyWestend as Chain>::RuntimeCall;

	let xcm = Xcm::<()>(vec![
		WithdrawAsset(assets),
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
			dest: bx!(VersionedLocation::V5(Parent.into())),
			message: bx!(VersionedXcm::V5(xcm)),
		});

	let _ =
		<FrequencyWestend as FrequencyWestendPallet>::Sudo::sudo(t.root_origin, bx!(inner_call));

	Ok(())
}

fn assert_receiver_minted_on_relay(t: FrequencyToRelayTest) {
	type RuntimeEvent = <Westend as Chain>::RuntimeEvent;
	let sov_frequency_on_relay =
		Westend::sovereign_account_id_of(Westend::child_location_of(FrequencyWestend::para_id()));

	Westend::assert_ump_queue_processed(
		true,
		Some(FrequencyWestend::para_id()),
		Some(Weight::from_parts(306305000, 7_186)),
	);

	assert_expected_events!(
		Westend,
		vec![
			// Amount to reserve transfer is withdrawn from Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Burned { who, amount }
			) => {
				who: *who == sov_frequency_on_relay.clone().into(),
				amount: *amount == t.args.amount,
			},
			RuntimeEvent::Balances(pallet_balances::Event::Minted { .. }) => {},
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

fn assert_sender_burned_asset_on_frequency(t: FrequencyToRelayTest) {
	type RuntimeEvent = <FrequencyWestend as Chain>::RuntimeEvent;
	// FrequencyWestend::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
	// 	864_610_000,
	// 	8_799,
	// )));
	assert_expected_events!(
		FrequencyWestend,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::ForeignAssets(
				pallet_assets::Event::Burned { asset_id, owner, balance, .. }
			) => {
				asset_id: *asset_id == WestendLocation::get(),
				owner: *owner == t.sender.account_id,
				balance: *balance == t.args.amount,
			},
		]
	);
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
// ========= Reserve Transfers - DOT Asset - Frequency<>Relay ===========
// =========================================================================
// RUST_BACKTRACE=1 RUST_LOG="events,runtime::system=trace,xcm=trace" cargo test tests::send_xcm_to_relay -- --nocapture
/// Tests reserve transfer of DOT from Frequency to Relay (Westend).
/// Ensures sovereign account on relay is debited and receiver is credited.
#[test]
fn send_xcm_to_relay() {
	let sender = FrequencyWestendSender::get();
	let receiver = WestendReceiver::get();
	let amount_to_send: Balance = WESTEND_ED * 1000;

	setup_parent_asset_on_frequency(sender.clone(), amount_to_send * 2);
	fund_sov_frequency_on_westend(amount_to_send * 2);

	// let root_account_id = HashedDescription::<
	// 	AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	// 	DescribeTerminus,
	// >::convert_location(&Here.into());
	// println!("something---------------- {:?}", root_account_id.unwrap());
	// // 0x0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8

	// mint_dot_on_frequency_v2(root_account_id.clone().unwrap(), amount_to_send);

	// let root_account_balance =
	// 	foreign_balance_on!(FrequencyWestend, Parent.into(), &root_account_id.clone().unwrap());
	// println!("root_account_balance({:?}-------- {:?}", root_account_id.unwrap(), root_account_balance);

	let assets: Assets = (Parent, amount_to_send).into();
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
	println!(
		"treasury_account_balance({:?})-------- {:?}",
		FrequencyTreasuryAccount::get(),
		treasury_account_balance
	);
	assert_eq!(treasury_account_balance, 0u128);

	let sender_assets_before = foreign_balance_on!(FrequencyWestend, dot_location.clone(), &sender);

	let receiver_balance_before = test.receiver.balance;

	// test.set_assertion::<FrequencyWestend>(assert_sender_burned_asset_on_frequency);
	// test.set_assertion::<Westend>(assert_receiver_minted_on_relay);
	test.set_dispatchable::<FrequencyWestend>(frequency_to_relay_send_xcm);
	test.assert();

	let treasury_account_balance = foreign_balance_on!(
		FrequencyWestend,
		dot_location.clone(),
		&FrequencyTreasuryAccount::get()
	);
	assert!(treasury_account_balance == 0u128, "Treasury account should NOT have been credited");
}
