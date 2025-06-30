// This file overrides the default Substrate CheckNonce for Frequency.
// It only creates the token account for paid extrinsics.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use frame_system::Config;
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode};

use frame_support::{
	dispatch::{DispatchInfo, Pays},
	sp_runtime,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		AsSystemOriginSigner, DispatchInfoOf, Dispatchable, One, TransactionExtension,
		ValidateResult,
	},
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionSource, TransactionValidityError,
		ValidTransaction,
	},
	Weight,
};
extern crate alloc;
use alloc::vec;

/// Nonce check and increment to give replay protection for transactions.
///
/// # Transaction Validity
///
/// This extension affects `requires` and `provides` tags of validity, but DOES NOT
/// set the `priority` field. Make sure that AT LEAST one of the signed extension sets
/// some kind of priority upon validating transactions.
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckNonce<T: Config>(#[codec(compact)] pub T::Nonce);

impl<T: Config> CheckNonce<T> {
	/// utility constructor. Used only in client/factory code.
	pub fn from(nonce: T::Nonce) -> Self {
		Self(nonce)
	}
}

impl<T: Config> core::fmt::Debug for CheckNonce<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "CheckNonce({})", self.0)
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}

impl<T: Config> TransactionExtension<T::RuntimeCall> for CheckNonce<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
	<T::RuntimeCall as Dispatchable>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
{
	const IDENTIFIER: &'static str = "CheckNonce";
	type Implicit = ();
	type Val = ();
	type Pre = ();

	fn weight(&self, _call: &T::RuntimeCall) -> Weight {
		// TODO: benchmark this
		Weight::zero()
	}

	fn validate(
		&self,
		origin: <T as Config>::RuntimeOrigin,
		_call: &T::RuntimeCall,
		_info: &DispatchInfoOf<T::RuntimeCall>,
		_len: usize,
		_self_implicit: Self::Implicit,
		_inherited_implication: &impl Encode,
		_source: TransactionSource,
	) -> ValidateResult<Self::Val, T::RuntimeCall> {
		// Only check for signed origin
		let Some(who) = origin.as_system_origin_signer() else {
			return Ok((ValidTransaction::default(), (), origin));
		};

		let account = frame_system::Account::<T>::get(&who);
		if self.0 < account.nonce {
			return Err(InvalidTransaction::Stale.into());
		}

		let provides = vec![Encode::encode(&(who, self.0))];
		let requires = if account.nonce < self.0 {
			vec![Encode::encode(&(who, self.0 - One::one()))]
		} else {
			vec![]
		};

		Ok((
			ValidTransaction {
				priority: 0,
				requires,
				provides,
				longevity: TransactionLongevity::MAX,
				propagate: true,
			},
			(),
			origin,
		))
	}

	fn prepare(
		self,
		_val: Self::Val,
		origin: &<T::RuntimeCall as Dispatchable>::RuntimeOrigin,
		_call: &T::RuntimeCall,
		info: &DispatchInfoOf<T::RuntimeCall>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		// Get TOKEN account from "who" key
		// Only check for signed origin
		let Some(who) = origin.as_system_origin_signer() else {
			return Ok(());
		};

		let mut account = frame_system::Account::<T>::get(&who);

		// The default account (no account) has a nonce of 0.
		// If account nonce is not equal to the tx nonce (self.0), the tx is invalid.  Therefore, check if it is a stale or future tx.
		if self.0 != account.nonce {
			return Err(if self.0 < account.nonce {
				InvalidTransaction::Stale
			} else {
				InvalidTransaction::Future
			}
			.into());
		}

		// Is this an existing account?
		// extracted from the conditions in which an account gets reaped
		// https://github.com/paritytech/polkadot-sdk/commit/e993f884fc00f359dd8bf9c81422c5161f3447b5#diff-dff2afa7433478e36eb66a9fe319efe28cfbdf95104b30b03afa0a1c4e3239f3R1082
		let existing_account =
			account.providers > 0 || account.consumers > 0 || account.sufficients > 0;

		// Increment account nonce by 1
		account.nonce += T::Nonce::one();

		// Only create or update the token account if the caller is paying or
		// account already exists
		if info.pays_fee == Pays::Yes || existing_account {
			frame_system::Account::<T>::insert(&who, account);
		}

		Ok(())
	}
}
