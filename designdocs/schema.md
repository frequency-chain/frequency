# Frequency Message Schemas

## Context and Scope

Messages on Frequency are validated and stored against pre-defined schema(s). In order to support a variety of message types, it is imperative to define an on-chain semantics, pallet(s) for example, to handle dynamic registration, validation, storage and retention schemes for schemas.

This document describes how schemas are handled on chain in the following sections.

## Problem Statement

Message passing is a core functionality to networking protocols. The way to enforce a communication protocol between participants of network via services is done by messaging schema. Analogous to interfaces, schemas provide a strongly typed description for messages ensuring their correctness, validity, extensibility and interoperability between services interacting with Frequency.

## Goals

At a minimum, Frequency should implement procedures to register, validate, store and access variety of messaging schemas dynamically. Schemas on chain must have the following salient features:

- **Registry**: Implement a schema registry, enabling participants to register and store validated schemas on chain.

- **Validation**: Schema validation enables message consumers and producers to entrust Frequency with correctness of schema prior to storage and a valid ```schema_id``` is produced. Schema validation can be done on chain with following basic steps:
  - Some sort of duplication checks to be put in place ensuring uniqueness of schemas.
  - Total count of schemas does not exceed a pre-defined maximum count that can be stored on chain.
  - Schema being registered should have a minimum size as defined by Frequency and should not exceed a pre-defined maximum size.
  - Schema should not be malformed.

Note: due to the [serialization concerns](./OnChainMessageStorage.md#serialization-concerns) pertaining to processing restrictions on chain as well as lack of better serialization rust libraries, schema integrity may be required to be validated off chain.

- **Interfaces**: Implement appropriate procedural calls to perform read operations on schema registry.

- **Retention**: Implement some sort of schema(s) retention logic for optimal on-chain message(s) storage. Retention periods per schema can be modified via super user permissions.

- **Schema Retirement**: Schema retirement is a mechanism to enable deprecation/retirement or invalidating a given schema from use. This can be achieved via defined schema states such as Active, Deprecated or Retracted. Rationale behind such a mechanism is as follows:

  - Author of a given schema would want to retire or deprecate a schema.
  - Schema itself has bug which was overlooked during registration.
  - Cost of garbage collecting data would eventually be a factor.

- **Evolution**: An important aspect of message passing is schema evolution. After initial schema is defined, network participants may need to evolve it over time depending on their respective use cases, it is critical to messaging system to handle data encoded with both old and new schema seamlessly. Schema evolution on Frequency can be achieved simply via various approaches, preferably some sort of retirement mechanism discussed in this proposal. See [additional notes](#additional-notes) for more details.

## Proposal

This document outlines various components of Frequency schemas, including, but not limited to, ability of network participants to register, validate and access message schemas dynamically via on chain semantics.

## Schema Registry

Schema registry provides an on chain repository for schemas, thereby allowing participants of the network to flexibly interact and exchange messages with each other without facing the challenge of sharing, managing and validating messages as well as schemas between them.

Using schema registry, message producers no longer need to include full schema with the message payload, instead only include ID of that schema, resulting in efficient serialization and storage.

![registry](https://user-images.githubusercontent.com/61435908/163263866-adf36d23-0968-42cd-8d50-6025bb7c455b.png)

### Schema Primitives

- **BlockNumber**: Chain specific primitive type for block number. Typically a 32-bit quantity.
- **Schema**: Serialized schema of type ```Vec<u8>```.
- **SchemaId**: A unique identifier of type ```u16``` for schemas that are successfully stored on chain.
- **BlockCount**: A primitive of type ```u16``` that represents count of blocks per schema. This is used to define for how many blocks; messages per schema are stored on chain.
- **SchemaState**: A type enumeration listing various state of a given schema[*](#disclaimer).

  ```rust

  pub  enum SchemaState {
    Active,
    Deprecated,
    Retracted,
  }

  ```

- **SchemaValidity**: Defines a contract enabling definition of state of current ```schema_id``` and its validity range in terms of block number. Typically, a generic schema validity can be defined as follows[*](#disclaimer):

  ```rust
  pub  struct SchemaValidity {
    pub state: SchemaState,
    pub valid_from: BlockNumber,
    pub valid_to: BlockNumber,
  }

  ```

- **SchemaPolicy** : Defines a contract that encapsulate ```retention``` which is of type ```BlockCount``` and ```starting_block``` which of type ```BlockNumber```. A typical generic structure for schema policy is defined as follows[*](#disclaimer):

  ```rust

  pub  struct SchemaPolicy {
    pub retention: BlockCount,
    pub starting_block: BlockNumber,
    pub validity: SchemaValidity
  }

  ```

### Schema Storage

- **Type definition**: ```StorageMap<_, Twox64Concat, SchemaId, BoundedVec<Schema,T::MaxSchemaSize>>```
- **Description**: Schemas are stored as key-value pair of SchemaId vs Serialized schema payload allowed to a maximum size.
- **Implementation**: Frequency will expose a substrate extrinsic ``` create_schema ``` to allow participants store a schema on chain. On successful registration raise ```SchemaCreated``` event with ```schema_id``` and schema payload. Schema registration should also initialize default ```SchemaPolicy``` upon successful schema registration.

### Schema Validation

Schema registry, at least, performs following checks before onboarding a schema on Frequency:

- Payload is signed by an authorizing AccountId.
- Frequency did not exceed maximum count of schemas that can be hosted on chain.
- A given schema adheres to minimum and maximum size limits allowed per schema.
- Schema itself is not broken or malformed.
- No duplicate schema(s) ever get registered.

### Schema Access

Schema registry should expose, at minimum, following procedural calls (as RPC and/or public trait for internal use) for network participants and off chain message validator. Depending on use case we might need to add more and modify these basic calls.

- **get_schema** : given a ```schema_id```, return serialized schema payload of type ```Schema``` stored on chain.

- **get_schema_state**: given a ```schema_id```, return the state and/or range of blocks between which the schema is valid, if it still exists on chain.

### Schema Retention and Starting Block Storage

Retention periods on a schema is designed for message(s) store to retain messages per schema to a specific block number. Retention periods can be updated via super user (sudo extrinsic on substrate) access.

- **Type Definition**: ```StorageMap<SchemaId, SchemaPolicy>```.
- **Description**: Retention period are stored as a map of ```SchemaId``` and ```SchemaPolicy```. By default schemas have no retention policy and by default ```retention``` and ```starting_block``` is set to 1 signaling message store to retain messages on chain database indefinitely.
- **Implementation**: Frequency will expose a substrate sudo call ```update_schema_retention``` to update ```retention``` period for a given ```schema_id```. On successful execution, retention block count will be updated.
- **Read**: Schema registry should expose ```get_retention_period``` procedural call to return current state of retention period for a given ```schema_id```.

Note: ```starting_block``` should only be modifiable via internal calls, for example, via message store and should not be exposed to consumers. Check out the following section for more details.

### Starting Blocks Storage and Access

- **Description**: On chain storage of starting block number for each schema. Required by message store. Defaults to block number 1.
- **Implementation**: Schema registry should provide some sort of procedural call (internal to Frequency) to read (```get_schema_starting_block```) and write (```set_schema_starting_block```) starting block number for a given ```schema_id```. This will be utilized by message store for further processing.
- **Rationale**: Message store periodically garbage collect messages per schema based on their retention period for on chain storage, upon successful garbage collection message store will update starting block to last block where messages were removed from on chain storage to chain database and new set of message will be store till ```starting_block + block_count-1```.

### Schema Retirement/Deprecation

Schema(s) being immutable on Frequency, would generally follow a cycle of deprecation/retirement for various reasons, such as, but not limited to, schema being wrong from consumer perspective , such as missing key fields that author intend to have and author would want to retire or deprecate a schema, even from chain perspective, the cost of garbage collection, processing feed or storage fees over time would require Frequency to regularly garbage collect stable/expired/deprecated schemas. In general, following salient features have been proposed to address schema retirement/deprecation:

  1. Schema(s) are immutable.
  2. Schema(s) that are intended to be retired based on their usage or vulnerabilities can be proposed to be deprecated in bulk via some sort of off chain governance.
  3. Same process proposed above can be used for bulk schema deletion for old/outdated schema(s) which are deemed to be not active anymore for example.
  4. Schema retirement/deprecation should be done in bulk via governance.

**Implementation**: ```SchemaValidity``` defines a generic structure of what encompasses a particular schema validity. Where ```SchemaState``` defines various stages of schema existence on chain. Schemas when registered should default to Active state. Some of the possible extrinsic calls that are required to realize this mechanism of schema retirement could be as follows[*](#disclaimer):

  1. ***Update*** ```update_schema_state```: Given a ```schema_id```. A valid account with sufficient balance can mark a schema deprecated or retracted (terms may change for how we want to word these). Such an update should be an outcome of curation via governance mechanism and hence can be implemented as a substrate sudo extrinsic. Typically we want to update state to ```Deprecated``` and  ```valid_from```, ```valid_to``` will be defined as range of blocks between which deprecated schema remains valid or in other words, a deprecation period.
  2. ***Delete***:```delete_schema```: Given a ```schema_id```. A valid account with sufficient balance can delete a schema from chain. Such an update should be an outcome of curation via governance mechanism and hence can be implemented as a substrate sudo extrinsic. Governance could also look at all the schemas that are past their validity range as discussed in 1. and decision around their deletion could be made, if not used anymore.

Note: Given the nature of dependency on governance we might want these extrinsic to be implemented as a sudo call. More on governance will be discussed in future.

### Schema Evolution

With Schema Registry, different consumers of Frequency can evolve a given schema at different rates, changing the shape of data and entrusting schema registry to handle translations from one schema to another. Currently schema evolution is not directly supported on chain and can be achieved by different consumers via unique [schema retirement procedure](#schema-retirementdeprecation) for evolved schemas. This is work in progress and various suggestions for future reference and development are listed in [additional notes](#additional-notes), preferably [suggestion 4](#suggestion-4).

## Benefits and Risks

### Benefits

- Schema registry allows message producers and consumers to efficiently share messages without having to store or manage schema text themselves.

- It allows message store to serialize, deserialize as well as optimize storage pattern of messages much more effectively.

- It also enable schema evolution and act as an interface to ensure contracts between consumers and producers is not broken, while promoting reusability of schemas.

- Schema immutability prevents overly complicated implementation and evolution while schema retirement via governance mechanism simplifies the process.

### Risks

- Schema registration on Frequency should prevent DoS attempts given schema registration will be open to anyone with enough balance to pay for the same. Should schema registration be costly, or restricting it specific accounts would be worth considering.

- Schema evolution is critical to any message passing system, how does Frequency intend to handle it or is it required , is still a question that needs to be ironed out.

- Another factor to consider is who is permissioned to modify retention periods per schema, who will pay for such an update and what are the defaults, if any.

- Is removing schema completely (deletion) a good idea? May be yes, for it is done via proper governance. No? ensuring right procedures to follow before deleting a schema.

- How we will handle duplicates, simple approach on chain or off chain. Not a risk but more of an implementation concern.

## Additional Resources

- [On chain message storage](./OnChainMessageStorage.md)
- [Substrate storage](https://docs.substrate.io/build/runtime-storage/)
- [Substrate extrinsics](https://docs.substrate.io/learn/transaction-types)
- [Substrate custom rpc](https://docs.substrate.io/build/remote-procedure-calls)
- [Substrate sudo](https://www.shawntabrizi.com/substrate/the-sudo-story-in-substrate/)

## Additional Notes

### Disclaimer

subject to change at implementation level, use it as a reference point.

### Schema Evolution Discussion

#### Suggestion 1

1. If we use a format that support schema evolution like Thrift or protobuf then we can basically replace existing schema with newer version which is backwards compatible with older one. here are risks and benefits

   - **risk**: we need to have a way preferably (on-chain) to validate if a new version is compatible with older version. I think we should look into this to see if it's possible or not.
   - **benefit**: if we are able to do this then then total number of existing schemas will get reduced and based on how the message retention policy works, with less schemas we will have more block capacity to handle more messages.

2. We just allow adding new schema for a new version of an old one.

   - **risk**: then number of schemas can get larger and would have more overhead on each block calculations due to retention policies of each schema.
   - **risk**: since read caching is also directly dependent on schema Id then more schemas will cause more overhead on cache calculations and storage
   - **benefit**: simpler process and no need to check for backwards compatibility

#### Suggestion 2

- Schemas should be immutable
- Why? Just because the schema creator wants to change something, doesn't mean the users do. Think of this as a limited version of smart contracts.
- Schemas should communicate replacement
- This is a different path than that of evolution, but I think we have three things we need to know:
  - 1. If the schema is replaced by a newer one.
  - 2. Which schema (or schemas?) replaced the old one.
  - 3. If the new schema is backward compatible with the old one.

- Should `#3` is needed to be communicated by the chain. Instead that feels like a library issue as to if the library can use the same code path or not for both.

- Should other items be communicated? If so by whom? The creator of the original schema or the replacement schema? I think that it is better to communicate on the fork than the original schema. It also allows it to be immutable.

#### Suggestion 3

- Schema(s) are immutable.
- Schema(s) can retire at different rates.
- Updated schemas can be added as long they have some sort of reference to previous/older schema(s).
- If a schema is retired, it should be propagated to referring schemas and removed from references.
- Provide some rpc calls for consumers to get replacement schemaId(s)/schema(s) for given schemaId etc.

#### Suggestion 4

Simple: Use retirement mechanism to have simpler less complicated evolution mechanism and make evolution synonym with schema retirement/deprecation.
