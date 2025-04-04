#[cfg(feature = "std")]
use crate::utils::as_hex;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_system::{EventRecord, Phase};
use parity_scale_codec::{Codec, Decode, Encode, EncodeLike};
use scale_info::TypeInfo;
extern crate alloc;
use alloc::vec::Vec;
use core::fmt::Debug;

/// The Struct for the getEvents RPC
/// This handles the Scale encoding of ONLY the event
/// Making a standardized and easy way to get events only caring about
/// the SCALE types of the events you care about
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, TypeInfo, Clone, Encode, Decode, PartialEq, Eq)]
pub struct RpcEvent {
	/// Which extrinsic call/Aka block index
	/// A None here means that it was in the Initialization or Finalization Phase
	pub phase: Option<u32>,
	/// Pallet index from the runtime
	pub pallet: u8,
	/// Index of the Event from the Pallet
	pub event: u8,
	/// The data of just the event SCALE Encoded
	#[cfg_attr(feature = "std", serde(with = "as_hex",))]
	pub data: Vec<u8>,
}

impl<RuntimeEvent, Hash> From<EventRecord<RuntimeEvent, Hash>> for RpcEvent
where
	RuntimeEvent: Codec + Sync + Send + TypeInfo + Debug + Eq + Clone + EncodeLike + 'static,
{
	fn from(event: EventRecord<RuntimeEvent, Hash>) -> Self {
		let phase = match event.phase {
			Phase::ApplyExtrinsic(i) => Some(i),
			_ => None,
		};

		// This requires that SCALE encoding does NOT change how it encodes u8s and indexes on enums
		let full_encoding = Encode::encode(&event.event);
		let pallet = full_encoding[0];
		let event = full_encoding[1];
		let data = &full_encoding[2..];

		RpcEvent { phase, pallet, event, data: data.to_vec() }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, TypeInfo, Clone, Encode, Decode, Eq, PartialEq)]
	enum FakeRuntime {
		#[codec(index = 60)]
		FakePallet(FakePalletEvents),
	}

	#[derive(Debug, TypeInfo, Clone, Encode, Decode, Eq, PartialEq)]
	enum FakePalletEvents {
		#[codec(index = 7)]
		FakeEvent { msa_id: u64 },
	}

	#[test]
	fn rpc_event_from_event_record() {
		let event: EventRecord<FakeRuntime, ()> = EventRecord {
			phase: Phase::ApplyExtrinsic(5),
			event: FakeRuntime::FakePallet(FakePalletEvents::FakeEvent { msa_id: 42 }),
			topics: vec![],
		};
		let rpc_event: RpcEvent = event.into();
		assert_eq!(
			rpc_event,
			RpcEvent { phase: Some(5), pallet: 60, event: 7, data: vec![42, 0, 0, 0, 0, 0, 0, 0] }
		);
	}

	#[test]
	fn rpc_event_from_event_record_with_different_phase() {
		let event: EventRecord<FakeRuntime, ()> = EventRecord {
			phase: Phase::Finalization,
			event: FakeRuntime::FakePallet(FakePalletEvents::FakeEvent { msa_id: 42 }),
			topics: vec![],
		};
		let rpc_event: RpcEvent = event.into();
		assert_eq!(
			rpc_event,
			RpcEvent { phase: None, pallet: 60, event: 7, data: vec![42, 0, 0, 0, 0, 0, 0, 0] }
		);
	}
}
