# MRC Message Schemas

## Context and Scope

Messages on MRC are validated and stored against pre-defined schema(s). In order to support a variety of message types, it is imperative to define an on-chain semantics, pallet(s) for example, to handle dynamic registration, validation, storage and retention schemes for schemas.
This document describes how schemas are handled on chain in the following sections.

## Problem Statement

Message passing is a core functionality to social networks. The way to enforce a communication protocol between participants of social network via services is done by messaging schema. Analogous to interfaces, schemas provide a strongly typed description for messages ensuring their correctness, validity, extensibility and interoperability between services interacting with MRC.

## Goals

At a minimum, MRC should implement procedures to register, validate, store and access variety of messaging schemas dynamically. Schemas on chain must have the following salient features:

- **Registry**: Implement a schema registry, enabling participants to register and store validated schemas on chain.
- **Validation**: Implement procedural calls enabling consumers to validate messages against stored schema. Due to serialization concerns [message validation](./OnChainMessageStorage.md) will be done off chain, while schema validation can be done on chain. Some basics of on chain validation required by MRC are as follows:
  - Total count of schemas does not exceed a pre-defined maximum count that can be stored on chain.
  - Schema being registered should have a minimum size as defined by MRC and should not exceed a pre-defined maximum size.
- **Interfaces**: Implement appropriate procedural calls to perform read operations on schema registry.
- **Retention**: Implement some sort of schema(s) retention logic  for optimal on-chain message(s) storage. Retention periods per schema can be modified via super user permissions.
- **Evolution**: An important aspect of message passing is  schema evolution. After initial schema is defined, network participants may need to evolve it over time depending of their respective use cases, this will be critical to messaging system to handle data encoded with both old and new schema seamlessly. This is a topic of research, if MRC would support schema evolution per se.

## Proposal

This document outlines various components of MRC schemas, including, but not limited to, ability of network participants to register, validate and access message schemas dynamically via on chain semantics.

## Schema Registry

Schema registry provides an on chain repository for schemas, thereby allowing participants of the network to flexibly interact and exchange messages with each other without facing the challenge of sharing, managing and validating messages as well as schemas between them.

Using schema registry, message producers no longer need to include full schema with the message payload, instead only include ID of that schema, resulting in efficient serialization and storage.

![registry](https://user-images.githubusercontent.com/61435908/163263866-adf36d23-0968-42cd-8d50-6025bb7c455b.png)

### Schema Primitives

- **Schema**: Serialized schema of type ```Vec<u8>```.
- **SchemaId**: A unique identifier of type ```u32``` for schemas that are successfully stored on chain.

### Schema Storage

- **Type definition**: ```StorageMap<_, Twox64Concat, SchemaId, BoundedVec<Schema,T::MaxSchemaSize>>```
- **Description**: Schemas are stored as key-value pair of SchemaId vs Serialized schema payload allowed to a maximum size.
- **Implementation**: MRC will expose a substrate extrinsic ``` register_schema ``` to allow participants store a schema on chain. On successful registration raise ```SchemaRegistered``` event with ```schema_id``` and schema payload.

### Schema Validation

Schema registry performs following checks before onboarding a schema on MRC:

- Ensure, payload is signed by an authorizing AccountId.
- Ensure, MRC did not exceed maximum count of schemas that can be hosted on chain.
- Ensure, a given schema adheres to minimum and maximum size limits allowed per schema.
- Ensure, schema itself is not broken or malformed.

### Schema Access

Schema registry should expose, at minimum, following procedural calls (as RPC and/or public trait for internal use) for network participants and off chain message validator.

- get_schema : given a ```schema_id```, return serialized schema payload of type ```Vec<Schema>``` stored on chain.

### Schema Retention

Retention periods on a schema is designed for message(s) store to retain messages per schema to a specific block number. Retention periods can be updated via super user (sudo extrinsic on substrate) access.

- **Type Definition**: ```StorageMap<SchemaId, BlockNumber>```.
- **Description**: Retention period are stored as a map of ```SchemaId``` and ```BlockNumber```. By default schemas have no retention policy and ```BlockNumber``` is set to 1 signaling message store to retain messages on chain database indefinitely.
- **Implementation**: MRC will expose a substrate  sudo call ```update_schema_retention``` to updated ```BlockNumber``` for a given ```schema_id```.
- **Read**: Schema registry should expose ```get_retention_period``` procedural call to return current state of retention period for a given ```schema_id```.

### Starting Blocks Storage and Access

- **Type Definition**: ```StorageMap<SchemaId, BlockNumber>```.
- **Description**: On chain storage of starting block number for each schema. Required by message store. Defaults to block number 1.
- **Implementation**: Schema registry should provide some sort of procedural call (internal to MRC) to read (```get_schema_starting_block```) and write (```set_schema_starting_block```) starting block number for a given ```schema_id```. This will be utilized by message store for further processing.

### Schema Evolution

With Schema Registry, different consumers of MRC can evolve a given schema at different rates, changing the shape of data and entrusting schema registry to handle translations from one schema to another. Currently schema evolution is not directly supported on chain and can be achieved by different consumers via unique ```schema_id``` for evolved schemas.

## Benefits and Risks

### Benefits

- Schema registry allows message producers and consumers to efficiently share messages without having to store or manage schema text themselves.
- It allows message store to serialize, deserialize as well as optimize storage pattern of messages much more effectively.
- It also enable schema evolution and act as an interface to ensure contracts between consumers and producers is not broken, while promoting reusability of schemas.

### Risks

- In general MRC can handle badly designed schemas via validation steps and punish participants registering bad schemas with the chain.
- Schema evolution is critical to any message passing system, how does MRC intend to handle it or is it required , is still a question that needs to be ironed out.
- Another factor to consider is who is permissioned to modify retention periods per schema, who will pay for such an update and what are the defaults, if any.

## Additional Resources

- [On chain message storage](./OnChainMessageStorage.md)
- [Substrate storage](https://docs.substrate.io/v3/runtime/storage/)
- [Substrate extrinsic](https://docs.substrate.io/v3/concepts/extrinsics/)
- [Substrate custom rpc](https://docs.substrate.io/v3/runtime/custom-rpcs/)
- [Substrate sudo](https://www.shawntabrizi.com/substrate/the-sudo-story-in-substrate/)
