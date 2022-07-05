use super::*;
use crate as pallet_tx_fee;
use codec::Encode;
use smallvec::smallvec;
use sp_core::H256;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use std::cell::RefCell;

use frame_support::{
	pallet_prelude::Get,
	parameter_types,
	traits::{ConstU32, ConstU64, Imbalance, OnUnbalanced},
	weights::{
		DispatchClass, GetDispatchInfo, Weight, WeightToFee as WeightToFeeT,
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use frame_system as system;
use pallet_balances::Call as BalancesCall;
use pallet_transaction_payment::{CurrencyAdapter, InclusionFee, Multiplier, NextFeeMultiplier};
use sp_runtime::{traits::SaturatedConversion, FixedPointNumber};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		MrcTxFeePallet: pallet_tx_fee::{Pallet},
	}
);

thread_local! {
	static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(0)
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow()).into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = 1024.into();
			})
			.build_or_panic()
	}
}

parameter_types! {
	pub static WeightToFee: u64 = 1;
	pub static TransactionByteFee: u64 = 1;
	pub static OperationalFeeMultiplier: u8 = 5;

}

impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Runtime {
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type LengthToFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = ();
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl WeightToFeeT for WeightToFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(*weight).saturating_mul(WEIGHT_TO_FEE.with(|v| *v.borrow()))
	}
}

impl WeightToFeeT for TransactionByteFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(*weight)
			.saturating_mul(TRANSACTION_BYTE_FEE.with(|v| *v.borrow()))
	}
}

thread_local! {
	static TIP_UNBALANCED_AMOUNT: RefCell<u64> = RefCell::new(0);
	static FEE_UNBALANCED_AMOUNT: RefCell<u64> = RefCell::new(0);
}

pub struct DealWithFees;
impl OnUnbalanced<pallet_balances::NegativeImbalance<Runtime>> for DealWithFees {
	fn on_unbalanceds<B>(
		mut fees_then_tips: impl Iterator<Item = pallet_balances::NegativeImbalance<Runtime>>,
	) {
		if let Some(fees) = fees_then_tips.next() {
			FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow_mut() += fees.peek());
			if let Some(tips) = fees_then_tips.next() {
				TIP_UNBALANCED_AMOUNT.with(|a| *a.borrow_mut() += tips.peek());
			}
		}
	}
}

impl Config for Runtime {}

pub struct ExtBuilder {
	balance_factor: u64,
	base_weight: u64,
	byte_fee: u64,
	weight_to_fee: u64,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balance_factor: 1, base_weight: 0, byte_fee: 1, weight_to_fee: 1 }
	}
}

impl ExtBuilder {
	pub fn base_weight(mut self, base_weight: u64) -> Self {
		self.base_weight = base_weight;
		self
	}
	pub fn weight_fee(mut self, weight_to_fee: u64) -> Self {
		self.weight_to_fee = weight_to_fee;
		self
	}
	fn set_constants(&self) {
		EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
		TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
		WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_constants();
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
		pallet_balances::GenesisConfig::<Runtime> {
			balances: if self.balance_factor > 0 {
				vec![
					(1, 10 * self.balance_factor),
					(2, 20 * self.balance_factor),
					(3, 30 * self.balance_factor),
					(4, 40 * self.balance_factor),
					(5, 50 * self.balance_factor),
					(6, 60 * self.balance_factor),
				]
			} else {
				vec![]
			},
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}
}

#[test]
fn compute_extrinsic_fee_works() {
	let call = Call::Balances(BalancesCall::transfer { dest: 2, value: 69 });
	let origin = 111111;
	let extra = ();
	let xt = TestXt::new(call.clone(), Some((origin, extra)));
	let info = xt.get_dispatch_info();
	let ext = xt.encode();
	let len = ext.len() as u32;

	let unsigned_xt = TestXt::<_, ()>::new(call, None);

	ExtBuilder::default().base_weight(5).weight_fee(2).build().execute_with(|| {
		// all fees should be x1.5
		<NextFeeMultiplier<Runtime>>::put(Multiplier::saturating_from_rational(3, 2));

		assert_eq!(
			MrcTxFeePallet::compute_extrinsic_cost(xt, len),
			FeeDetails {
				inclusion_fee: Some(InclusionFee {
					base_fee: 5 * 2,
					len_fee: len as u64,
					adjusted_weight_fee: info.weight.min(BlockWeights::get().max_block) as u64 *
						2 * 3 / 2
				}),
				tip: 0,
			},
		);

		assert_eq!(
			MrcTxFeePallet::compute_extrinsic_cost(unsigned_xt, len),
			FeeDetails { inclusion_fee: None, tip: 0 },
		);
	});
}
