use crate::xcm::{tests::mock::UniversalLocation, Barrier};
use frame_support::weights::Weight;
use staging_xcm::{opaque::v3::MultiLocation, prelude::*};
use xcm_executor::traits::{Properties, ShouldExecute as XcmBarrier};

#[test]
fn test_barrier_allows_parent() {
	let location = Location::parent();
	let mut instructions =
		Xcm::<()>(vec![TransferAsset { assets: (Parent, 100).into(), beneficiary: Here.into() }]);
	let mut properties =
		Properties { weight_credit: Weight::from_parts(1_000_000_000, 0), message_id: None };
	let weight = Weight::from_parts(1_000_000_000, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.inner_mut(),
		weight,
		&mut properties,
	);

	assert!(result.is_ok(), "Barrier test failed: {:?}", result);
}

#[test]
fn test_barrier_fails_unauthoized_location_unpaid() {
	let location = Parachain(3001).into();
	let mut instructions = Xcm::<()>(vec![UnpaidExecution {
		weight_limit: Limited(Weight::from_parts(20, 20)),
		check_origin: Some(Location::parent()),
	}]);
	let mut properties = Properties { weight_credit: Weight::zero(), message_id: None };
	let weight = Weight::from_parts(10, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.inner_mut(),
		weight,
		&mut properties,
	);
	assert!(result.is_err(), "Barrier test failed: {:?}", result);
}

#[test]
fn test_barrier_allows_parent_location_unpaid() {
	let location = Location::parent();
	let mut instructions = Xcm::<()>(vec![UnpaidExecution {
		weight_limit: Limited(Weight::from_parts(20, 20)),
		check_origin: Some(Location::parent()),
	}]);
	let mut properties = Properties { weight_credit: Weight::zero(), message_id: None };
	let weight = Weight::from_parts(10, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.inner_mut(),
		weight,
		&mut properties,
	);
	assert!(result.is_ok(), "Barrier test failed: {:?}", result);
}

#[test]
fn test_barrier_denies_unpaid_random_location() {
	let location = Location::new(100, []);
	let mut instructions =
		Xcm::<()>(vec![TransferAsset { assets: (Parent, 100).into(), beneficiary: Here.into() }]);
	let mut properties = Properties { weight_credit: Weight::zero(), message_id: None };
	let weight = Weight::from_parts(1_000_000_000, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.inner_mut(),
		weight,
		&mut properties,
	);

	assert!(
		result.is_err(),
		"Barrier should deny execution for random location, but it allowed: {:?}",
		result
	);
}

#[test]
fn test_barrier_allows_paid_random_location() {
	let location = Location::new(100, []);
	let mut instructions =
		Xcm::<()>(vec![TransferAsset { assets: (Parent, 100).into(), beneficiary: Here.into() }]);
	let mut properties =
		Properties { weight_credit: Weight::from_parts(1_000_000_000, 0), message_id: None };
	let weight = Weight::from_parts(1_000_000_000, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.inner_mut(),
		weight,
		&mut properties,
	);

	assert!(
		result.is_ok(),
		"Barrier should allow execution for paid random location, but it denied: {:?}",
		result
	);
}
