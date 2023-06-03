// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

mod parachains;
mod relay_chain;

pub mod foreign_chain_alias_account;
pub mod with_computed_origin;

use foreign_chain_alias_account::*;
use codec::Encode;
use polkadot_core_primitives::AccountId;
use parachains::{frequency, parachain};
use polkadot_parachain::primitives::Id as ParaId;
use sp_runtime::traits::AccountIdConversion;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};
use sp_io::hashing::blake2_256;
use frame_support::parameter_types;

#[cfg(test)]
mod tests;


pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 2_000_000_000;

pub fn alice_foreign_alias_account() -> sp_runtime::AccountId32 {
	sp_runtime::AccountId32::new((FOREIGN_CHAIN_PREFIX_PARA_32, 1u32, &[0u8; 32], 1u8).using_encoded(blake2_256))
}


decl_test_parachain! {
	pub struct ParaA {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(1),
	}
}

decl_test_parachain! {
	pub struct ParaB {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(2),
	}
}

decl_test_parachain! {
	pub struct Frequency {
		Runtime = frequency::Runtime,
		XcmpMessageHandler = frequency::MsgQueue,
		DmpMessageHandler = frequency::MsgQueue,
		new_ext = {
			use frequency::{MsgQueue, Runtime, System};

			let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();


			// let alice_foreign_alias_account = (FOREIGN_CHAIN_PREFIX_PARA_32, 1u32, &[0u8; 32], 1u8).using_encoded(blake2_256);
			pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, INITIAL_BALANCE), (para_account_id(1), INITIAL_BALANCE), (alice_foreign_alias_account().into(), INITIAL_BALANCE)] }
				.assimilate_storage(&mut t)
				.unwrap();

			let mut ext = sp_io::TestExternalities::new(t);
			ext.execute_with(|| {
				System::set_block_number(1);
				MsgQueue::set_para_id(3u32.into());
			});

			ext
		},
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relay_chain::Runtime,
		XcmConfig = relay_chain::XcmConfig,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1, ParaA),
			(2, ParaB),
			(3, Frequency),
		],
	}
}

pub fn para_account_id(id: u32) -> relay_chain::AccountId {
	ParaId::from(id).into_account_truncating()
}

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use parachain::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE), (para_account_id(1), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use relay_chain::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE), (para_account_id(1), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type FrequencyPalletXcm = pallet_xcm::Pallet<frequency::Runtime>;
