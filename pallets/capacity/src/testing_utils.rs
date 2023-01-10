use super::*;
use crate::mock::*;
use frame_support::{
	assert_ok,
	traits::{ConstU32, OnFinalize, OnInitialize},
	BoundedVec,
};

pub fn register_provider(target_id: MessageSourceId, name: String) {
	let name: BoundedVec<u8, ConstU32<16>> = Vec::from(name).try_into().expect("error");
	assert_ok!(Msa::create_registered_provider(target_id.into(), name));
}

pub fn staking_events() -> Vec<Event<Test>> {
	let result = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(
			|e| if let mock::RuntimeEvent::Capacity(inner) = e { Some(inner) } else { None },
		)
		.collect::<Vec<_>>();

	System::reset_events();
	result
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Capacity::on_initialize(System::block_number());
	}
}
