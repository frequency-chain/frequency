use crate::imports::*;
use emulated_integration_tests_common::{
	test_parachain_is_trusted_teleporter_for_relay, test_relay_is_trusted_teleporter,
};

#[test]
fn teleport_from_and_to_relay() {
	let amount = WESTEND_ED * 100;
	let native_asset: Assets = (Here, amount).into();

	test_relay_is_trusted_teleporter!(
		Westend,
		WestendXcmConfig,
		vec![FrequencyWestend],
		(native_asset, amount)
	);

	test_parachain_is_trusted_teleporter_for_relay!(
		FrequencyWestend,
		FrequencyWestendXcmConfig,
		Westend,
		amount
	);
}
