use crate::xcm::Barrier;
use frame_support::weights::Weight;
use staging_xcm::{opaque::v3::MultiLocation, prelude::*};
use xcm_executor::traits::{Properties, ShouldExecute as XcmBarrier};

#[test]
fn test_barrier_allows_parent_exec_plurality_free() {
	let location = Location::new(1, []);
	let mut instructions =
		vec![Instruction::<()>::ClearOrigin, Instruction::DescendOrigin(Junctions::Here)];
	let mut properties =
		Properties { weight_credit: Weight::from_parts(1_000_000_000, 0), message_id: None };
	let weight = Weight::from_parts(1_000_000_000, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.as_mut_slice(),
		weight,
		&mut properties,
	);

	assert!(result.is_ok(), "Barrier test failed: {:?}", result);
}

#[test]
fn test_barrier_denies_random_location() {
	let location = Location::new(100, []);
	let mut instructions =
		vec![Instruction::<()>::ClearOrigin, Instruction::DescendOrigin(Junctions::Here)];
	let mut properties = Properties { weight_credit: Weight::zero(), message_id: None };
	let weight = Weight::from_parts(1_000_000_000, 0);

	let result = <Barrier as XcmBarrier>::should_execute(
		&location,
		instructions.as_mut_slice(),
		weight,
		&mut properties,
	);

	assert!(
		result.is_err(),
		"Barrier should deny execution for random location, but it allowed: {:?}",
		result
	);
}
