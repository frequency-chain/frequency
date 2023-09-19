use common_primitives::msa::MsaValidator;
use frame_support::traits::tokens::Balance;
use sp_std::marker::PhantomData;

use super::*;
use crate::Config;

/// A trait used for the withdrawal of Capacity.
pub trait OnChargeCapacityTransaction<T: Config> {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// Handles withdrawal of Capacity from an Account.
	fn withdraw_fee(
		key: &T::AccountId,
		fee: Self::Balance,
	) -> Result<Self::Balance, TransactionValidityError>;
}

/// A type used to withdraw Capacity from an account.
pub struct CapacityAdapter<Curr, Msa>(PhantomData<(Curr, Msa)>);

impl<T, Curr, Msa> OnChargeCapacityTransaction<T> for CapacityAdapter<Curr, Msa>
where
	T: Config,
	Curr: Currency<<T as frame_system::Config>::AccountId>,
	Msa: MsaValidator<AccountId = <T as frame_system::Config>::AccountId>,
	BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<CapacityBalanceOf<T>> + MaxEncodedLen,
{
	type Balance = BalanceOf<T>;

	/// Handle withdrawal of Capacity using a key associated to an MSA.
	/// It attempts to replenish an account of Capacity before withdrawing the fee.
	fn withdraw_fee(
		key: &T::AccountId,
		fee: Self::Balance,
	) -> Result<Self::Balance, TransactionValidityError> {
		ensure!(
			Curr::free_balance(key) >= Curr::minimum_balance(),
			//ChargeFrqTransactionPaymentError::BelowMinDeposit.into()
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);

		let msa_id = Msa::ensure_valid_msa_key(key)
			.map_err(|_| ChargeFrqTransactionPaymentError::InvalidMsaKey.into())?;

		if T::Capacity::can_replenish(msa_id) {
			ensure!(
				T::Capacity::replenish_all_for(msa_id).is_ok(),
				//ChargeFrqTransactionPaymentError::TargetCapacityNotFound.into()
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);
		}

		ensure!(
			T::Capacity::deduct(msa_id, fee.into()).is_ok(),
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);

		Ok(fee)
	}
}
