
# Capacity design doc

## Context and Scope

In this document I will introduce the concept of Capacity which will provide a way for users to obtain fee-less transactions. 

Feeless transactions are important in reaching mass adoption as it removes the overhead costs of transactions for app developers to acquire a far reaching user base.

**What is Capacity?**

Capacity is a non-transferable resource that is associated to an MSA account. You can use capacity to pay for transactions and, what makes it unique, is that it replenishes after a certain period which can be used to continue to pay for new transactions. More details on how capacity replenishes below.

## Proposal

MRC has two types of account systems—a MSA account and a token account. You can acquire Capacity by taking your own tokens to your MSA account or when others stake their tokens to your MSA. The minimum amount required for staking  is 2x existential deposit plus fees. The  amount staked is reserved from the users account free balance which is refunded after an unstacking period. More details on staking and rewards are left for another design doc.

Once staked,  Capacity is granted at a ratio  6,000 token to 1 unit of capacity.  This can then be used to pay for transactions. 

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

   fn available(msa_id: MessageSenderId) -> Result<Balance, DispatchError>;
   fn reduce_available(msa_id: MessageSenderId, amount: Balance) -> Result<Balance, DispatchError>;
   fn increase_available(msa_id: MessageSenderId, amount: Balance) -> Result<Balance, DispatchError>;
 }

```

We can represent Capacity as a mapping of an MsaId to information containing the total balance available to an MSA.

```rust
#[pallet::storage]
pub type CapacityOf<T: Config> = StorageMap<_, Blake2_128Concat, MessageSenderId, Details<T::Balance>>;

pub struct Details {
	pub used: Balance,
	pub available: Balance,
}
```

And we can store the a token account staking to MSA by mapping.

```rust
#[pallet::storage]
pub type Staking<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, StakingDetails<T::Balance>>;

pub struct StakingDetails<Balance> {
	pub msa_id: MessageSenderId,
	pub total_reserved: Balance,
}
```

To give the limitations of what you can do with Capacity you are limited to staking and un-staking Capacity for token. Note that these interface can change as we introduce stash and controller accounts using Proxy pallet.

```rust
pub fn stake(origin: OriginFor<T>, account: MessageSenderId, amount: BalanceOf<T>)

pub fn stake(origin: OriginFor<T>, account: MessageSenderId, amount: BalanceOf<T>)
```

Events for triggers for staking include

```rust
pub enum Event<T: Config> {
  Staked { account: T::AccountId, msa_id: MessageSenderId, amount: BalanceOf<T> },

  UnStake{ account: T::AccountId, msa_id: MessageSenderId, amount: BalanceOf<T> },
}
```

Errors when staking

```rust
pub enum Error<T> {
	AlreadyStaked,
	InsufficientBalance,		
	InvalidMsa,
}
```

**Capacity is replenishable**

Replenishable mean that after a certain fixed period, called an epoch, all capacity is replenished. The epoch period is composed of  blocks. A block is filled with 25% transactions being operational and 75% for all other transactions including Capacity. This can be configurable by setting the  `AvailableBlockRatio` . Moreover, one unit of Capacity is `10^12` and matches with substrate weight measurement where `10^12` is 1 second of execution time.

Since we want the ability to replenish Capacity the following interface is implement to replenish capacity.

```rust

traits Replenishable {
   type Balance: Balance;
   
	 fn replenish_by_account(msa_id: MessageSenderId, amount: Balance) -> Result<Balance, Eroor> {};
   fn replenish_all_for_account(msa_id: MessageSenderId) -> Result<Balance, Error>;
   fn can_replenish(msa_id: MessageSenderId) -> bool;
}
```

**How capacity replenishes?**

As mentioned above, epoch periods are composed of blocks. The next epoch period calculated  is based on the current epoch fullness, and expands and contracts depending on a threshold. This threshold is configurable and can be called `config::epochThreshold` . Additionally, we can also configure a multiplier that we increase or decrease the next epoch based on the fullness of the current epoch. This is defined as `config::epochUtilizationMultiplier`.

![Untitled](https://s3-us-west-2.amazonaws.com/secure.notion-static.com/e74b133a-60ae-4ec8-af5e-046240d208b9/Untitled.png)

The above illustrates two epochs where the second one contracts because network congestion has decreased. As a result of epoch decreasing, capacity is replenish faster.

**How are epochs calculated?**

Epoch are used to manage congestion on the network. As demand increases for network resources the epoch length increases which results in capacity taking longer to replenish. 

Upon the finalization of each block, we can get the total amount of weight used and update the total amount of weight for an epoch.

![Untitled](https://s3-us-west-2.amazonaws.com/secure.notion-static.com/8c016520-dd60-4402-b4c3-7ac51db24c76/Untitled.png)

**Transactions fees**

When submitting transactions with Capacity before including these transaction in a block they are validated at the transaction pool. The validation implemented doing a `SignedExtension` that validates that the signed transaction is associated to an MSA that contains a Capacity balance. If the MSA has capacity it then withdraws from the available balance and increases the amount of capacity is used. If no Capacity is available than it removes the transaction from the pool.

## Non-Goals

- collator incentives
- Governance
    
    

## Benefits and Risk

- rewards

## Alternatives and Rationale

- all MSA get tokens

## Glossary

