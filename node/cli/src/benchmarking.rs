//! Setup code for [`super::command`] which would otherwise bloat that module.
//!
//! Should only be used for benchmarking as it may break in other contexts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use common_primitives::node::{AccountId, Balance, Signature};
use frame_support::pallet_prelude::InherentData;
use frame_system::{Call as SystemCall, Config};
use frequency_service::service::{frequency_runtime as runtime, ParachainClient as FullClient};
use sc_cli::Result;
use sc_client_api::BlockBackend;
use sp_core::{Encode, Pair};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};

use pallet_balances::Call as BalancesCall;
use sp_inherents::InherentDataProvider;
#[allow(deprecated)]
use sp_runtime::traits::transaction_extension::AsTransactionExtension;
use std::{sync::Arc, time::Duration};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
	client: Arc<FullClient>,
}

impl RemarkBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<FullClient>) -> Self {
		Self { client }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
	fn pallet(&self) -> &str {
		"system"
	}

	fn extrinsic(&self) -> &str {
		"remark"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			SystemCall::remark { remark: vec![] }.into(),
			nonce,
		)
		.into();

		Ok(extrinsic)
	}
}

/// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct TransferKeepAliveBuilder {
	client: Arc<FullClient>,
	dest: AccountId,
	value: Balance,
}

impl TransferKeepAliveBuilder {
	/// Creates a new [`Self`] from the given client.
	pub fn new(client: Arc<FullClient>, dest: AccountId, value: Balance) -> Self {
		Self { client, dest, value }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for TransferKeepAliveBuilder {
	fn pallet(&self) -> &str {
		"balances"
	}

	fn extrinsic(&self) -> &str {
		"transfer_keep_alive"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			BalancesCall::transfer_keep_alive { dest: self.dest.clone().into(), value: self.value }
				.into(),
			nonce,
		)
		.into();

		Ok(extrinsic)
	}
}

/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic(
	client: &FullClient,
	sender: sp_core::sr25519::Pair,
	call: runtime::RuntimeCall,
	nonce: u32,
) -> runtime::UncheckedExtrinsic {
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;

	let period = <runtime::Runtime as Config>::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;

	#[allow(deprecated)]
	let extra: runtime::TxExtension = cumulus_pallet_weight_reclaim::StorageWeightReclaim::<runtime::Runtime, _>::new(
		(
			frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
			(
				frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
				frame_system::CheckTxVersion::<runtime::Runtime>::new(),
			),
			frame_system::CheckGenesis::<runtime::Runtime>::new(),
			frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
				period,
				best_block.saturated_into(),
			)),
			#[allow(deprecated)]
			AsTransactionExtension::from(common_runtime::extensions::check_nonce::CheckNonce::<runtime::Runtime>::from(nonce)),
			#[allow(deprecated)]
			AsTransactionExtension::from(pallet_frequency_tx_payment::ChargeFrqTransactionPayment::<runtime::Runtime>::from(0)),
			#[allow(deprecated)]
			AsTransactionExtension::from(pallet_msa::CheckFreeExtrinsicUse::<runtime::Runtime>::new()),
			#[allow(deprecated)]
			AsTransactionExtension::from(pallet_handles::handles_signed_extension::HandlesSignedExtension::<runtime::Runtime>::new()),
			frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
			frame_system::CheckWeight::<runtime::Runtime>::new(),
		),
	);
	let raw_payload = sp_runtime::generic::SignedPayload::from_raw(
		call.clone(),
		extra.clone(),
		(
			(),
			(runtime::VERSION.spec_version, runtime::VERSION.transaction_version),
			genesis_hash,
			best_hash,
			(),
			(),
			(),
			(),
			None,
			(),
		),
	);
	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	runtime::UncheckedExtrinsic::new_signed(
		call.clone(),
		sp_runtime::AccountId32::from(sender.public()).into(),
		Signature::Sr25519(signature),
		extra.clone(),
	)
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> Result<InherentData> {
	let mut inherent_data = InherentData::new();
	let d = Duration::from_millis(0);
	let timestamp = sp_timestamp::InherentDataProvider::new(d.into());
	let mock_para_inherent_provider =
		cumulus_client_parachain_inherent::MockValidationDataInherentDataProvider {
			para_id: 1000.into(),
			current_para_block_head: Some(cumulus_primitives_core::relay_chain::HeadData::default()),
			current_para_block: 0,
			relay_offset: 1,
			relay_blocks_per_para_block: 1,
			xcm_config: Default::default(),
			raw_downward_messages: Default::default(),
			raw_horizontal_messages: Default::default(),
			para_blocks_per_relay_epoch: 2,
			relay_randomness_config: (),
			additional_key_values: Some(vec![]),
			upgrade_go_ahead: None,
		};

	futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
		.map_err(|e| format!("creating inherent data: {:?}", e))?;
	futures::executor::block_on(
		mock_para_inherent_provider.provide_inherent_data(&mut inherent_data),
	)
	.map_err(|e| format!("creating cumulus inherent data: {:?}", e))?;

	Ok(inherent_data)
}
