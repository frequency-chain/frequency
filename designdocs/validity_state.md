# Validity States

## Table of Contents (Optional)
* [Glossary](#glossary)
* [Context and Scope](#context-and-scope)
* [Problem Statement](#problem-statement)
* [Goals and Non-Goals](#goals-and-non-goals)
* [Proposal](#proposal)
* [Benefits and Risks](#benefits-and-risks)
* [Alternatives and Rationale](#alternatives-and-rationale)
* [Additional Resources](#additional-resources)

## Glossary (Optional)
* **term**: definition

## Context and Scope
In several discussions about features of MRC we've talked about how to prevent retirement, deletion, or other changes in validity of storage data in MRC from causing inappropriate validation failures of old messages.

This proposal suggests a general way to handle validity state for a given piece of data.

## Problem Statement
We have realized that MsaIds, Message Schemas, and Delegations need the ability to be rendered unusable for messages beyond a specific time (block) without necessarily making previous messages impossible to validate.  These needs arose for different reasons, however they all point to a need for at minimum basic validity state data, and rudimentary state machine behavior.

## Goals and Non-Goals
This proposal aims to:
* specify a common data structure for validity state storage
* suggest how to incorporate this validity state into validation for three types of data in MRC.
* specify an API for registering and updating schema
* suggest a migration path

This proposal does not aim to specify:
* what the states are
* what the validity state structure is
* how they are stored on chain (what Storage type to use)

There may be some storage types and choices of keys that are better than others and these should be determined by experiments.

## Proposal

### State Rules
For maximum efficiency, lowest churn and least conflict, there must be restrictions on the state transitions.

The rules are summarized by saying:
* there's no going backward,
* there's one and only one possible state for a given block number,
* the highest state value is invalid and final and can't ever be changed,
* ...except in a migration, in which case it must be migrated to be the final state of the new migration.


1. The number of states and state transitions must be strictly limited to something humanly small, e.g. less than 100 or 1000.
100 or 1000 states may seem like a lot, however, it may not be when considering state type migrations as described later on.
2. Validity state type is an enumeration, starting with Active. The state Active is the default and has the lowest state value of all the states in use.
3. Successive states should be in a sensible order of expected progression, with a final state being an invalid state.
4. Final states must be set with their end block = 0.
5. States may not change from a higher value to a lower value.
6. Validity states have a block range where the state applies, a start and an end.
7. The block range end default value is 0, which means the state validity is indefinite. The reason for this (rather than it being -1 which is often used to indicate the last index) is `Blocknumber` has a type of u128.
8. The validity end block _must_ be 0 for the current state.
9. When a new state is applied for block N, the previous state's block end MUST be set to N-1.
10. State block ranges may not overlap.  Example:
    1. Schema 124 is registered with 1000, 0 and given the default, Active
    2. Schema 124 is updated with Deprecated,, 4999, 0
    3. When queried, the validity state returns
        ```rust
        [ (SchemaState::Active: 1000,4998), (SchemaState::Deprecated, 4999,0) ]
       ```
11. To wipe out the validity range for an ID's previous state, the state is overwritten with the new validity and range.  Only one previous state may be overwritten.  The state may be overwritten only with a higher value state.

    **Example 1**
     1. Schema 345 is registered with start of 1000 and end of 0, state Active.
     2. The registrar for Schema 345 submits a new state update, "retracted", with a start of 1000 and an end of 0.
     3. When queried, the validity state returns
       ```rust
          [ (Retracted, 1000,0) ]
       ```
     4. The registrar tries to submit a new state of `Active` with block 1000, 0.  This fails.

    **Example 2**
    1. Schema 456 is registered with start of 1000 and end of 0, state `Active`
    2. Schema 456 is updated to Deprecated,, 4999,0
    3. When queried, the validity state returns
    ```rust
        [ (Active, 1000,4998), (Deprecated, 4999, 0) ] }
    ```
    3. Registrar tries to submit a new state, Retracted, 1000, 0.  This fails.

### Structure
Enums can automatically derive Debug, PartialEq, and PartialOrd traits for ease of state comparison, and for serialization, which would let us keep storage size to a minimum.  Similarly for deserialization
the `std::convert::From` trait could be implemented to interpret state updates.
```rust
use std::fmt;
#[derive(Debug, PartialEq, PartialOrd)]
enum SchemaState {
    Active,
    Deprecated,
    Retracted
}

impl fmt::Display for SchemaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

fn main() {
    use SchemaState::*;
    assert!(Active < Deprecated);
    assert!(Deprecated < Retracted);

    println!("{}", Deprecated);
}

```
```
------------------ Standard Output --------------------
Deprecated
```

### Delegation states
Delegation states would be different. A delegator may wish to reinstate a delegate. Deprecation doesn't make sense for a delegation relationship. So the states might be:
Active, Retracted, "reinstated", "terminated".  If we decided that Delegates need a couple of chances then the states could always be set as Active, Retracted, "reinstated", "retracted2", "reinstated2", "terminated", etc.


### Storage
Since querying will largely be to determine validity, it may bethat validity state is best stored in the main storage for MsaIds, Schemas, and Delegations, however, the goal here is not to specify how these will be stored or even what pallet they belong in.

With that said, validity states for each type of data should probably be kept separate from each other.

### API (extrinsics)

#### register_schema(`schema`, `msa_id`)
The schema registration API does not change, however, in storage it should now set the validity state to be the default (active and lowest possible in value) state, the start block = 'now' and the end block = 0.

#### update_schema(`schema_id`, `new_state`, `start_block`)
Adds a new validity state to the schema registry entry for `schema_id` with a block range of `start_block` to 0.  The current state end_block is set to `start_block - 1`.

**Parameters**

   1. `schema_id`: the `SchemaId` to update.
   2. `new_state`: the new state of the schema.
   3. `start_block`: when this state should go into effect.

### API (Custom RPC)
#### schema_state(`schema_id`)
Returns the validity states for the given `schema_id`. Returns `None()` if `schema_id` does not exist.  Example format:
```rust
    Some([ (Active: 1000,4998), (Deprecated, 4999, 0) ] )
```

#### delegation_state(`delegate_id`, `delegator_id`)
Returns the validity states for the given `delegate_id` and `delegator_id`. Returns `None()` if `schema_id` does not exist. Example format:

```rust
    Some([ (Active: 1000,4998), (Retired, 4999, 0) ] )
```

### schema_states() / delegation_states() / msa_id_states() -> ([]&str, uint32)
Returns all possible states (as strings), and the index of the default state

```rust
    (["Active", "Deprecated", "Retracted"], 0)
```

### State Validity Migration
An important issue to address is Validity State migration. The above rules are extremely strict. Permanently locking in the allowable states could pose serious risks for supporting future needs.

One potential solution to this is to add the new states to the set of possible states while keeping the old ones, setting a new default to be the start of the set of new states. Example:
1. Let's say that allowed `MsaId` states are currently `[Active,Deleted]`, with `Active` being the default.
2. It becomes evident that more states are needed, so a new set of states is applied:  `["active, Deleted, Active2, Paused, Retired, Deleted2]`.  The new default is `Active2`. By some decision-making process (such as through governance), it's determined everyone with `Active` will be set to `Active2`, everyone `Deleted` will be set to `Deleted2`, and the new states are now available to everyone at `Active2`.
3. The migration is applied to the validity state storage.

It's assumed that State Validity Migrations are rare in blockchain time.  Limiting the states to something humanly low would encourage developers to think and plan carefully about their needed states.

## Benefits and Risks
A benefit of a validity state storage system standardizes validation of messages. This gives consistency to data handling. Secondly it fulfills the goals of being able to specify different treatment for messages depending on the state of the different pieces of data.

One risk is that a "one size fits all" solution often winds up being "one size fits most", with the exceptions being difficult to work around. This solution is intended only for MsaIds, Message Schemas, and Delegations. Future data types must be considered as to whether they fit this paradigm and whether it's worth modifying it or creating a custom solution for the new data type.

Another risk is an increase in storage size will significantly impact performance and costs of running a node.

Most of the risks center around Schema states; a third issue is that this solution doesn't allow for schema evolution; if a schema is deprecated then it simply can't be used for new messages, and the new schema version simply must be re-registered.

At the same time, this restriction simplifies handling of schema changes.  Rather than using versioning, the Schema ID is the de facto version number. Old schemas remain valid for messages posted within their stored block range. Consumers of messages posted by the Schema owner will know what the Schema ID is for a given batch and will not need to try to track down the new Schema ID from that Announcer.

Furthermore, schema changes will emit new Events with needed informaiton and consumers will know when the schema is updated.

Since this is on a blockchain, and block number is a type of universally agreed-upon timestamp, there will be no uncertainty as to whether a message is valid for its associated schema id.  Messages are guaranteed to be part of one and only one batch, which is guaranteed to be announced on one and only one block.

## Alternatives and Rationale

## Additional Resources

* [Source name](http://www...) with description
