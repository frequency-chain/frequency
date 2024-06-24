use pallet_transaction_payment::OnChargeTransaction;

/// Type aliases used for interaction with `OnChargeTransaction`.
pub type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;

/// Balance type alias.
pub type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;

/// Liquidity info type alias (imbalances).
pub type LiquidityInfoOf<T> =
	<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;
