# Capacity Design Doc

## Context and Scope

Feeless transactions are important in reaching mass adoption as it removes the overhead costs of transactions for app developers to acquire a far reaching user base.

In this document I will introduce the concept of Capacity and how capacity can be used to pay for transaction fees.

**What is Capacity?**

Capacity is a non-transferable resource that is associated to an MSA account. You can use capacity to pay for transactions and it replenishes after a certain period which can be used to continue to pay for new transactions. More details on how capacity replenishes below.

## Proposal

MRC has two types of account systems—a MSA account and a token account. You can acquire Capacity by staking your own tokens to your MSA account or when others stake their tokens to your MSA. The amount staked is reserved from an account free balance which is refunded after a thaw period. Once staked, Capacity is granted at a 1 to 1 conversion.  Upon receiving Capacity, it can then be used to pay for transactions. 

There is a minimum required for staking that will be detailed further in a supplemental design that goes into the details of staking and rewards.

**Below** **are transactions that can be paid with Capacity:**

- Create an MSA
- Add a key to an MSA
- Delegate permissions to another MSA
- Update Delegate Permissions
- Send a message
- Send a batch message


Capacity will be stored as a mapping of an MsaId to information containing the total balance available to an MSA.

```rust
/// Storage for an MSA's Capacity balance details.
#[pallet::storage]
pub type CapacityOf<T: Config> = StorageMap<_, Blake2_128Concat, MessageSourceId, CapacityDetails<T::Balance>>;

pub struct CapacityDetails<Balance> {
  /// The amount of Capacity used for an Epoch.
  pub used: Balance,

  /// The total Capacity an MSA account has per Epoch.
  pub total_available: Balance,

  /// The last Epoch that an MSA was replenished.
  pub last_replenished_epoch: u32,

  /// The first Epoch for which Capacity was issued.
  pub initial_epoch: u32
}
```

And we can store records of a token account staking to MSA by using the following map.

```rust
/// Storage for keeping record of Accounts staking to MSA's.
#[pallet::storage]
pub type Staking<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, StakingDetails<T::Balance>>;

pub struct StakingDetails<Balance, BlockNumber> {
  /// The message source id to which an account is staked to.
	pub msa_id: MessageSourceId,
 
  /// The amount being staked to an MSA account.
	pub total_reserved: Balance,

  /// start of staking
	pub since: BlockNumber,
}
```

Initially, you are limited to staking and un-staking token for Capacity. As mentioned above a more detailed design doc around staking and rewards are deferred to the [staking design doc](https://github.com/LibertyDSNP/frequency/issues/40). Note that these interface can change as we introduce stash and controller accounts using Proxy pallet.

The amount staked is reserved from the account balance using the [NamedReserveableCurrency](https://paritytech.github.io/substrate/master/frame_support/traits/trait.NamedReservableCurrency.html) trait.



```rust
/// Stakes some amount of tokens to an MSA account.
/// 
/// ### Errors
/// 
/// - Returns Error::AlreadyStaked if sender already staked to that MSA.
/// - Returns Error::InsufficientBalance if sender does not have enought to cover the amount wanting to stake.
/// - Returns Error::InvalidMsa if attempting to stake to a non-existing MSA account.
pub fn stake(origin: OriginFor<T>, account: MessageSourceId, amount: BalanceOf<T>) -> DispatchResult {

}

/// Unstakes some amount of tokens from an MSA.
pub fn unstake(origin: OriginFor<T>, account: MessageSourceId, amount: BalanceOf<T>) -> DispatchResult {

}
```

Events for triggers for staking include

```rust
pub enum Event<T: Config> {
  /// A token account has staked to an MSA.
  Staked { 
    /// The token account that staked to an MSA.
    account: T::AccountId,
  
    /// The MSA that a token account staked too.
    msa_id: MessageSourceId,

    /// An amount that was staked.
    amount: BalanceOf<T>
  },

  /// A token account has unstaked to an MSA.
  UnStake{
    /// The token account that staked to an MSA.
    account: T::AccountId,

    /// The MSA that a token account unstaked too.
    msa_id: MessageSourceId,

    /// An amount that was unstaked.
    amount: BalanceOf<T>
   },
}
```

Errors when staking

```rust
pub enum Error<T> {
  /// Staker tried to stake to the same account more than once.
  AlreadyStaked,

  /// Staker does not have sufficient balance to cover the amount wanting to stake.
  InsufficientBalance,

  /// Staker attempted to stake to an invalid MSA.
  InvalidMsa,
}
```

As mentioned above, Capacity is non-transferable and at a implements the following interface to reduce and increase capacity on an MSA.

```rust
traits Nontransferable {
   type Balance = Balance;

   /// The available Capacity for an MSA account. 
   fn available(msa_id: MessageSourceId) -> Result<Balance, DispatchError>;
   
   /// Reduce the available Capacity of an MSA account. 
   fn reduce_available(msa_id: MessageSourceId, amount: Balance) -> Result<Balance, DispatchError>;

   /// Increase the available Capacity for an MSA account.
   fn increase_available(msa_id: MessageSourceId, amount: Balance) -> Result<Balance, DispatchError>;
 }

```

**Capacity is replenishable**

Replenishable means that after a certain fixed period, called an epoch, all capacity is replenished. The epoch period is composed of blocks.  Moreover, one unit of Capacity is `10^12` and matches with substrate weight measurement where `10^12` is 1 second of execution time.

Since we want the ability to replenish Capacity the following interface is implement to replenish capacity.

```rust
trait Replenishable {
  type Balance: Balance;
   
  /// Replenish an MSA amount.
  fn replenish_by_account(msa_id: MessageSourceId, amount: Balance) -> Result<Balance, DispatchError> {};

  /// Replish all available balance.
  fn replenish_all_for_account(msa_id: MessageSourceId) -> Result<Balance, DispatchError>;

  /// Checks if an account is able to be replenished.
  fn can_replenish(msa_id: MessageSourceId) -> bool;
}
```

**How capacity replenishes**

As mentioned above, epoch periods are composed of blocks. The next epoch period calculated is based on the current epoch fullness, and expands and contracts depending on a threshold. This threshold is configurable and can be called `config::epochThreshold`. Additionally, we can also configure a multiplier that we increase or decrease the next epoch based on the fullness of the current epoch. This is defined as `config::epochUtilizationMultiplier`.

![Untitled](https://user-images.githubusercontent.com/3433442/171747512-651112be-bbdc-4197-96ef-0cd208f702db.png)

The above illustrates two epochs where the second one contracts because network congestion has decreased. As a result of epoch decreasing, capacity is replenish faster. Note that initially epoch periods will be fixed in sized for a period of time.

**How are epochs calculated?**

Epoch are used to manage congestion on the network. As demand increases for network resources the epoch length increases which results in capacity taking longer to replenish. 

Upon the finalization of each block, we can get the total amount of weight used and update the total amount of weight for an epoch.

![Untitled](https://user-images.githubusercontent.com/3433442/171747526-566ee44a-194f-47e3-8abb-e2c415ac7fb5.png)

**Transaction fees**

When submitting signed extrinsics, before including these transaction in a block they are validated at the transaction pool. The validation implemented doing a [SignedExtension](https://docs.rs/sp-runtime/latest/sp_runtime/traits/trait.SignedExtension.html) that validates that signer has either a token or capacity balance. 

Since we have introduce an additional form of payment for a transactions via Capacity we cannot use FRAME's Transaction-Payment-Pallet. Instead we can create a wrapper around this pallet that allows us to toggle between different forms of payments. 

We can distinguish how to retrive if a transaction should be paid with Capacity or Token by requiring that a payment via Capacity to be called with `pay_with_capacity` extrinsic. 

```rust
#[pallet::call]
	impl<T: Config> Pallet<T> {
    #[pallet::weight({
		let dispatch_info = call.get_dispatch_info();
			(T::WeightInfo::pay_with_capacity().saturating_add(dispatch_info.weight), dispatch_info.class,)
		})]
		pub fn pay_with_capacity(
			origin: OriginFor<T>,
			call: Box<CallOf<T>>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin.clone())?;
			call.dispatch(origin)
		}
  }
```

`ChargeTransactionPayment` struct type is used to implement a SignedExtension which validates that the signer has enought Capacity or Token to withdraw a fee. A withdraw_fee method is used to resolve and withdraw payment fee from either a Token or Capacity account. If the signer does not have enought to pay for transaction errors with a `TransactionValidityError` and drops the transaction from the pool. 

```rust
/// A type that is used for implementing a SignedExtension to handle transaction fees validation and withdrawal.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPayment<T: Config>(#[codec(compact)] BalanceOf<T>);

impl<T: Config> ChargeTransactionPayment<T>
where
  CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>> + From<CallOf<T>>,
  BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<ChargeCapacityBalanceOf<T>>,
{
  /// utility constructor. Used only in client/factory code.
  pub fn from(fee: BalanceOf<T>) -> Self {
    Self(fee)
  }
  
  /// Given a fee is required, withdraw the fee from Capacity or Token account.
  fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &CallOf<T>,
		info: &DispatchInfoOf<CallOf<T>>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
    let tip = self.0;
    let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, tip);

    // Handle case when no fee is required.
    if fee.is_zero() {
      return Ok((fee, InitialPayment::Nothing))
    }
    
    // match against the call to be dispatched.
    match call.is_sub_type() {
      Some(Call::pay_with_capacity { .. }) =>
        T::OnChargeCapacityTransaction::withdraw_fee(1, call, info, fee.into(), tip.into())
          .map(|i| (fee, InitialPayment::Capacity(i))),
      _ => <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
        who, call, info, fee, self.0,
      )
      .map(|i| (fee, InitialPayment::Token(i)))
      .map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() }),
    }
  }
}
```

Below is the partial implementation of the SignedExtension on ChargeTransactionPayment containing the validate and pre-dispatch requirements.

```rust
/// Implement signed extension SignedExtension to validate that a transaction payment can be withdrawn for a Capacity or Token account. This allows transactions to be dropped from the transaction pool if signer does not have enough to pay for fee. Pre-dispatch withdraws the actual payment from the account and Post-dispatch refunds over charges made at pre-dispatch.
impl<T: Config + Send + Sync> SignedExtension for ChargeTransactionPayment<T>
where
	BalanceOf<T>: Send + Sync + FixedPointOperand + From<u64> + IsType<ChargeCapacityBalanceOf<T>>,
	CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
{
  const IDENTIFIER: &'static str = "ChargePayment";
	type AccountId = T::AccountId;
	type Call = CallOf<T>;
	type AdditionalSigned = ();
	type Pre = (BalanceOf<T>, Self::AccountId, InitialPayment<T>);

  /// Validate that payment can be withdrawn from Capacity or Token account.
  fn validate(
    &self,
    who: &Self::AccountId,
    call: &Self::Call,
    info: &DispatchInfoOf<CallOf<T>>,
    len: usize,
  ) -> TransactionValidity {
    use pallet_transaction_payment::ChargeTransactionPayment;
    
    let (fee, _) = self.withdraw_fee(who, call, info, len)?;
    let priority = ChargeTransactionPayment::<T>::get_priority(len, info, fee);
    
    Ok(ValidTransaction { priority, ..Default::default() })
  }

  /// Check and Withdraw fee payment from Capacity or Token account before executing authoring.
  fn pre_dispatch(
    self,
    who: &Self::AccountId,
    call: &Self::Call,
    info: &DispatchInfoOf<Self::Call>,
    len: usize,
  ) -> Result<Self::Pre, TransactionValidityError> {
    let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;

    let tip = self.0;
    Ok((tip, who.clone(), initial_payment))
  }

  ...
}

Notice that Pre-dispatch returns a type `Pre`, this is the type that gets passed from Pre-dispatch to Post-dispatch. It lets Post-dispach know how the payment was paid for in Capacity, Token or No payment.

```rust
/// Types for handling how the payment will processed. This type is passed to Post-dispatch to be able to distinguish how to reinbused overcharges in fee payment.
#[derive(Encode, Decode, DefaultNoBound, TypeInfo)]
pub enum InitialPayment<T: Config> {
  /// Pay no fee.
  Nothing,
  /// Pay fee with Token.
  Token(LiquidityInfoOf<T>),
  /// Pay fee with Capacity.
  Capacity(ChargeCapacityBalanceOf<T>),
}
```

When after the transaction is authored Post-dispatch refunds the overcharged Capacity or Token payment. Using the type `Pre` that gets passed in from Pre-dispatch it corrects the fee refunding the amount overcharged.

```rust
impl<T: Config + Send + Sync> SignedExtension for ChargeTransactionPayment<T>
where
	BalanceOf<T>: Send + Sync + FixedPointOperand + From<u64> + IsType<ChargeCapacityBalanceOf<T>>,
	CallOf<T>: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
{
  ...

	fn post_dispatch(
		pre: Self::Pre,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<CallOf<T>>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		use pallet_transaction_payment::ChargeTransactionPayment;

		let (tip, who, initial_payment) = pre;
		let actual_fee = pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
			len as u32, info, post_info, tip,
		);

		match initial_payment {
			InitialPayment::Token(already_withdrawn) => {
				let pre_pre = (tip, who, already_withdrawn);
				ChargeTransactionPayment::<T>::post_dispatch(
					pre_pre, info, post_info, len, result,
				)?;
			},
			InitialPayment::Capacity(already_withdrawn) => {
				let account = 1u32;
				T::OnChargeCapacityTransaction::correct_and_deposit_fee(
					&account,
					info,
					post_info,
					actual_fee.into(),
					tip.into(),
					already_withdrawn,
				)?;
			},
			InitialPayment::Nothing => {
				debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
			},
		}
		Ok(())
	}
```

If the MSA has capacity it then increases the amount of capacity is used. If no Capacity is remaining for the epoch period the transaction remains in the future queue until the next epoch. Once the next epoch starts and Capacity becomes available the transaction is processed.


Non-capacity transaction remained unchanged and follow the same default flow. During implementation a wrapper capacity transaction pallet is used to wrap pallet transaction payment to toggle between capacity and non-capacity transactions and set the [validity of the transaction.](https://paritytech.github.io/substrate/master/sp_runtime/transaction_validity/struct.ValidTransaction.html)

## Non-Goals
Staking details are left for another design document.

## Benefits and Risk
The benefits of implementing Capacity is that it allows applications to increase their users my reducing cost.

## Alternatives and Rationale



