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

As mentioned above, Capacity is non-transferable and at a minimum implements the following interface.

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

Capacity will be stored as a mapping of an MsaId to information containing the total balance available to an MSA.

```rust
/// Storage for an MSA's Capacity balance details.
#[pallet::storage]
pub type CapacityOf<T: Config> = StorageMap<_, Blake2_128Concat, MessageSourceId, Details<T::Balance>>;

pub struct CapacityDetails {
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

And we can store the a token account staking to MSA by mapping.

```rust
/// Storage for keeping record of Accounts staking to MSA's.
#[pallet::storage]
pub type Staking<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, StakingDetails<T::Balance>>;

pub struct StakingDetails<Balance> {
  /// The message source id to which an account is staked to.
	pub msa_id: MessageSourceId,
 
  /// The amount being staked to an MSA account.
	pub total_reserved: Balance,
}
```

Initially, you are limited to staking and un-staking token for Capacity. As mentioned above a more detailed design doc around staking and rewards are deferred to the [staking design doc](https://github.com/LibertyDSNP/frequency/issues/40). Note that these interface can change as we introduce stash and controller accounts using Proxy pallet.

```rust
/// Stakes some amount of tokens to an MSA account.
/// 
/// ### Errors
/// 
/// - Returns Error::AlreadyStaked if sender already staked to that MSA.
/// - Returns Error::InsufficientBalance if sender does not have enought to cover the amount wanting to stake.
/// - Returns Error::InvalidMsa if attempting to stake to a non-existing MSA account.
pub fn stake(origin: OriginFor<T>, account: MessageSourceId, amount: BalanceOf<T>)

/// Unstakes some amount of tokens from an MSA.
pub fn unstake(origin: OriginFor<T>, account: MessageSourceId, amount: BalanceOf<T>)
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

**Capacity is replenishable**

Replenishable means that after a certain fixed period, called an epoch, all capacity is replenished. The epoch period is composed of blocks.  Moreover, one unit of Capacity is `10^12` and matches with substrate weight measurement where `10^12` is 1 second of execution time.

Since we want the ability to replenish Capacity the following interface is implement to replenish capacity.

```rust
traits Replenishable {
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

As mentioned above, epoch periods are composed of blocks. The next epoch period calculated is based on the current epoch fullness, and expands and contracts depending on a threshold. This threshold is configurable and can be called `config::epochThreshold` . Additionally, we can also configure a multiplier that we increase or decrease the next epoch based on the fullness of the current epoch. This is defined as `config::epochUtilizationMultiplier`.

![Untitled](https://user-images.githubusercontent.com/3433442/171747512-651112be-bbdc-4197-96ef-0cd208f702db.png)

The above illustrates two epochs where the second one contracts because network congestion has decreased. As a result of epoch decreasing, capacity is replenish faster. Note that initially epoch periods will be fixed in sized for a period of time.

**How are epochs calculated?**

Epoch are used to manage congestion on the network. As demand increases for network resources the epoch length increases which results in capacity taking longer to replenish. 

Upon the finalization of each block, we can get the total amount of weight used and update the total amount of weight for an epoch

![Untitled](https://user-images.githubusercontent.com/3433442/171747526-566ee44a-194f-47e3-8abb-e2c415ac7fb5.png)

**Transactions fees**

When submitting capacity transactions, before including these transaction in a block they are validated at the transaction pool. The validation implemented doing a [SignedExtension](https://docs.rs/sp-runtime/latest/sp_runtime/traits/trait.SignedExtension.html) that validates that the signed transaction is associated to an MSA that contains a Capacity balance. If the MSA has capacity it then increases the amount of capacity is used. If no Capacity is remaining for the epoch period the transaction remains in the future queue until the next epoch. Once the next epoch starts and Capacity becomes available the transaction is processed.

Non-capacity transaction remained unchanged and follow the same default flow. During implementation a wrapper capacity transaction pallet is used to wrap pallet transaction payment to toggle between capacity and non-capacity transactions and set the [validity of the transaction.](https://paritytech.github.io/substrate/master/sp_runtime/transaction_validity/struct.ValidTransaction.html)

![Untitled](https://user-images.githubusercontent.com/3433442/171749900-21470787-b74f-44fa-b32d-ac06138f7616.png)


## Non-Goals
Staking details are left for another design document.

## Benefits and Risk
The benefits of implementing Capacity is that it allows applications to increase their users my reducing cost.

## Alternatives and Rationale



