use crate::xcm::parameters::HereLocation;
use crate::xcm::{queue::PriceForSiblingParachainDelivery, xcm_config::XcmConfig};
use crate::{ExistentialDeposit, ParachainSystem};
use frame_support::parameter_types;
use cumulus_primitives_core::ParaId;
use polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper;
use staging_xcm::opaque::latest::Asset;

parameter_types! {
	pub ExistentialDepositAsset: Option<Asset> = Some((
		HereLocation::get(),
		ExistentialDeposit::get()
	).into());

	pub const RandomParaId: ParaId = ParaId::new(17171717);
}

pub type ParachainDeliveryHelper = ToParachainDeliveryHelper<
	XcmConfig,
	ExistentialDeposit,
	PriceForSiblingParachainDelivery,
	RandomParaId,
	ParachainSystem,
>;
