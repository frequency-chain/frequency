# Capacity Design Doc

## Context and Scope

Feeless transactions are important in reaching mass adoption as it removes the overhead costs of transactions for app developers to acquire a far reaching user base.

In this document I will introduce the concept of [Capacity](https://forums.projectliberty.io/t/05-what-is-capacity-frequency-economics-part-1/248), a non-transferable resource that is associated to an MSA account of a [Registered Provider](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_registration.md),  and how Capacity, can be acquired through staking, replenished and used to perform certain transaction such as: 

- Create an MSA
- Add a key to an MSA
- Delegate permissions to another MSA
- Update Delegate Permissions
- Send a message
- Send a batch message

## Proposal

Frequency explains how Capacity can be acquired through staking, and replenished and used to perform certain transactions.  This approach is addressed in each section below:  

- [Implementation of how to acquire through staking](https://www.notion.so/Design-Doc-Capacity-00752bdabb8d420d97a126ccffcfb7a2)
- [Implementation of how to replenish Capacity](https://www.notion.so/Design-Doc-Capacity-00752bdabb8d420d97a126ccffcfb7a2)
- Implementation of using Capacity to perform transactions

**Implementation of how to acquire through staking:**

This section is limited to the interfaces for staking and unstaking tokens.  

As a Registered Provider, you can receive Capacity by staking your tokens to the network or when others stake their tokens to the network.

When staking tokens to the network, the network generates Capacity based on a Capacity-generating-function that takes into consideration usage and other criteria. When you stake tokens, you will also provide a target Registered Provider that will receive the Capacity generated. In exchange for staking token to the network, you receive rewards.  Rewards are deferred to a supplemental [staking design doc](https://github.com/LibertyDSNP/frequency/issues/40).

### **Interfaces for Staking-Pallet**

### **Calls**

**Stake**

```rust
/// Stakes some amount of tokens to the network and generates Capacity.
///
/// ### Errors
///
/// - Returns Error::AlreadyStaked if sender already staked to that MSA.
/// - Returns Error::InsufficientBalance if sender does not have enought to cover the amount wanting to stake.
/// - Returns Error::InvalidMsa if attempting to stake to a non-registered provider MSA account.
pub fn stake(origin: OriginFor<T>, account: MessageSourceId, amount: BalanceOf<T>) -> DispatchResult {}
```

Acceptance Criteria is listed below but can evolve:

1. Dispatched origin is Signed by Staker.
2. A Target MSA account must be a Registered Provided.
3. A token amount staked must not be greater than the free balance.
4. A Staker can only target an MSA of a Registered Provider.
5. A Staker can only target one Registered Provider at a time.
6. The token amount staked is to remain [locked](https://paritytech.github.io/substrate/master/frame_support/traits/trait.LockableCurrency.html) with reason [WithdrawReasons::all()](https://paritytech.github.io/substrate/master/frame_support/traits/tokens/struct.WithdrawReasons.html#method.all).
7. Capacity is generated with a configurable capacity-generating-function.
8. Target Registered Provider is issued generated Capacity.
9. Target Registered Provider issued Capacity becomes available at the start of the next Epoch Period.

**Unstake**

```rust
/// Unstakes some amount of tokens from an MSA.
///
/// ### Errors
///
/// - Returns Error::UnstakedAmountIsZero Unstaking amount should be greater than zero.
pub fn unstake(origin: OriginFor<T>, account: MessageSourceId, amount: BalanceOf<T>) -> DispatchResult {}
```

1. Dispatched origin is Signed by Staker.
2. Amount unstaked must be larger than 0.
3. Balance is unlocked after a configurable `UnstakingThawPeriod` (approximately 7 days).
4. Issued Capacity to Registered Provider is reduced after the next Epoch Period.
5. Amount unstaked cannot exceed the amount bonded.

### **Errors**

```rust
pub enum Error<T> {
  /// Staker tried to stake to the same account more than once.
  AlreadyStaked,
  /// Staker does not have sufficient balance to cover the amount wanting to stake.
  InsufficientBalance,
  /// Staker attempted to stake to an invalid MSA.
  NotRegisteredMsa,
  /// Staking amount is below minimum amount required.
  BelowMinStakingAmount,
  /// Unstaking amount should be greater than zero.
	UnstakedAmountIsZero,
  /// Account address is not staking
	NotStakingAccount,
  /// Unstaking amount is greatar than total contributed.
  AmountUnstakingExceedsBalance
}
```

### **Events**

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

### **Storage**

**Staking Storage**

Storage for keeping records of staking accounting:

```rust
/// Storage for keeping record of Accounts staking to MSA's.
#[pallet::storage]
pub type StakingLedger<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, StakingDetails<T::Balance>>;
```

The type for storing information about staking details:

```rust
pub struct StakingInfo<Balance, BlockNumber> {
  /// The message source id to which an account is staked to.
  pub msa_id: MessageSourceId,
  /// The amount being staked to an MSA account.
  pub total: Balance,
  /// start of staking
  pub since: BlockNumber,
}
```

### **Interfaces for Capacity-Pallet**

**Capacity Storage**

Storage for the issuance of Capacity to Registered Providers:

```rust
/// Storage for an MSA's Capacity balance details.
#[pallet::storage]
pub type CapacityOf<T: Config> = StorageMap<_, Blake2_128Concat, MessageSourceId, CapacityDetails<T::Balance>>;
```

The type for storing Registered Provider Capacity balance:

```rust
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

### **Traits**

As mentioned above, Capacity is non-transferable and implements the following interface to reduce and increase capacity on an MSA.

```rust
traits Nontransferable {
   type Balance;

   /// The available Capacity for an MSA account.
   fn available(msa_id: MessageSourceId) -> Result<Balance, DispatchError>;

   /// Reduce the available Capacity of an MSA account.
   fn reduce_available(msa_id: MessageSourceId, amount: Balance) -> Result<Balance, DispatchError>;

   /// Increase the available Capacity for an MSA account.
   fn increase_available(msa_id: MessageSourceId, amount: Balance) -> Result<Balance, DispatchError>;
 }
```

**Implementation of how to Replenish**

Replenishable means that after a certain fixed period, called an Epoch Period, all Capacity is replenished. An epoch period is composed of a fixed number of blocks.

![Untitled](https://s3-us-west-2.amazonaws.com/secure.notion-static.com/46da4934-02f5-41f4-912a-db3aeae91fb6/Untitled.png)

At the beginning of a new Epoch period Capacity is automatically replenished when making your first Capacity transaction in the new Epoch Period. 

![Untitled](https://s3-us-west-2.amazonaws.com/secure.notion-static.com/d2d2ae64-6ff5-4cc6-9410-1d6397f6050d/Untitled.png)

The following interface are implement on Capacity-Pallet to facilitate replenishment:

### **Hooks**

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	fn on_initialize(_now: BlockNumberFor<T>) -> Weight {}
}
```

1. Initializes a new Epoch-Period at the when Epoch Period ends.
2. When starting an new Epoch it increments `EpochNumber` 
3. Resets the `CurrentBlockUsedCapacity` storage.
4. On the start of a new block it resets `CurrentBlockUsedCapacity`

### Traits

Replenishable trait implemented on Capacity-Pallet facilitates replenishing Registered Provider Capacity.

```rust
trait Replenishable {
  type Balance;

  /// Replenish an MSA amount.
  fn replenish_by_account(msa_id: MessageSourceId, amount: Balance) -> Result<Balance, DispatchError> {};

  /// Replish all available balance.
  fn replenish_all_for_account(msa_id: MessageSourceId) -> Result<Balance, DispatchError>;

  /// Checks if an account is able to be replenished.
  fn can_replenish(msa_id: MessageSourceId) -> bool;
}

```

**Storage**

To help keep count of the Epoch numbers:

```rust
/// Storage for keep count of the number epoch
#[pallet::storage]
pub type EpochNumber<T> = StorageValue<_, u32, ValueQuery>;
```

To facilitate keeping track of how much Capacity was used for an Epoch Period.

```rust
/// Storage to keep accounting for the total of Capacity used in an Epoch.
#[pallet::storage]
pub type CurrentEpochUsedCapacity<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;
```

To facilitate keeping track of how much Capacity was consumed in a block.