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

//! Trappist Parachain runtime mock.

use common_primitives::node::{AccountId, Balance, BlockNumber, Hash, Header, Index};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{EitherOfDiverse, Everything, Nothing},
	weights::constants::RocksDbWeight,
};
use frame_system::EnsureRoot;

pub use common_runtime::{
	constants::{
		currency::EXISTENTIAL_DEPOSIT, CollatorKickThreshold, CollatorMaxCandidates,
		CollatorMaxInvulnerables, CollatorMinCandidates, NeverDepositIntoId,
	},
	fee::WeightToFee,
};

pub use frequency_runtime::{
	xcm_config::{
		Barrier, LocationToAccountId, MaxInstructions, RelayNetwork, SelfReserve, UnitWeightCost,
	},
	BalancesMaxLocks, BalancesMaxReserves, RuntimeBlockLength, RuntimeBlockWeights, Session,
	Version,
};
use pallet_xcm::{EnsureXcm, IsMajorityOfBody};
use polkadot_runtime_common::BlockHashCount;
use sp_core::{ConstU128, ConstU16, ConstU32};
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256};
use sp_std::prelude::*;
use xcm::latest::prelude::*;
use xcm_builder::{
	CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, IsConcrete, LocationInverter,
	SiblingParachainAsNative, SignedToAccountId32, UsingComponents,
};
use xcm_executor::{Config, XcmExecutor};

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl super::mock_msg_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = BalancesMaxLocks;
	type MaxReserves = BalancesMaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}
/// We allow root and the Relay Chain council to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureXcm<IsMajorityOfBody<RelayLocation, ExecutiveBody>>,
>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = NeverDepositIntoId;
	type MaxCandidates = CollatorMaxCandidates;
	type MinCandidates = CollatorMinCandidates;
	type MaxInvulnerables = CollatorMaxInvulnerables;
	type KickThreshold = CollatorKickThreshold;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = pallet_collator_selection::weights::SubstrateWeight<Runtime>;
}

// impl pallet_sudo::Config for Runtime {
// 	type RuntimeCall = RuntimeCall;
// 	type RuntimeEvent = RuntimeEvent;
// }

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;

	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;

	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;

	type XcmRouter = XcmRouter;

	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmExecuteFilter = Everything;

	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;

	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;

	type LocationInverter = LocationInverter<Ancestry>;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

parameter_types! {
	// pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
}

pub type AssetTransactors = (LocalAssetTransactor,);
pub type LocalAssetTransactor =
	CurrencyAdapter<Balances, IsConcrete<SelfReserve>, LocationToAccountId, AccountId, ()>;

pub type XcmOriginToTransactDispatchOrigin = (
	// RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
);
pub type XcmRouter = crate::ParachainXcmRouter<MsgQueue>;

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;

	type IsReserve = ();
	type IsTeleporter = ();

	type LocationInverter = LocationInverter<Ancestry>;

	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;

	type Trader = (UsingComponents<WeightToFee, SelfReserve, AccountId, Balances, ()>,);

	type ResponseHandler = ();

	type AssetTrap = ();
	type AssetClaims = ();

	type SubscriptionService = ();

	type XcmSender = XcmRouter;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		MsgQueue: super::mock_msg_queue::{Pallet, Storage, Event<T>},
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 31,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
		// Sudo: pallet_sudo = 40,
	}
);
