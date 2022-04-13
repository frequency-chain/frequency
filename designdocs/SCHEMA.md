# MRC Message Schemas

## Context and Scope

Messages on MRC are validated and stored against pre-defined schema(s). In order to support a variety of message types, it is imperative to define an on-chain semantics, pallet(s) for example, to handle dynamic registration, validation, storage and retention schemes for schemas.
This document describes how schemas are handled on chain in the following sections.

## Problem Statement

Message passing is a core functionality to social networks. The way to enforce a communication protocol between participants of social network via services is done by messaging schema. Analogous to interfaces, schemas provide a strongly typed description for messages ensuring their correctness, validity, extensibility and interoperability between services interacting with MRC.

## Goals

At a minimum, MRC should implement procedures to register, validate, store and access variety of messaging schemas dynamically. Schemas on chain must have the following salient features:

- **Registry**: Implement a schema registry, enabling participants to register and store validated schemas on chain.
- **Validation**: Implement procedural calls enabling consumers to validate messages against stored schema. Due to serialization concerns message validation will be done off chain, while schema validation can be done on chain. Some basics of on chain validation required by MRC are as follows:
  - Total count of schemas does not exceed a pre-defined maximum count that can be stored on chain.
  - Schema being registered should have a minimum size as defined by MRC and should not exceed a pre-defined maximum size.
- **Interfaces**: Implement appropriate procedural calls to perform CRUD operations on schema registry.
- **Retention**: Implement some sort of schema(s) retention logic  for optimal on-chain storage. Retention periods per schema can be modified via super user permissions.
- **Evolution**: TODO

## Proposal

This document outlines various components of MRC schemas, including, but not limited to, ability of network participants to register, validate and access message schemas dynamically via on chain semantics.

## Schema Registry

Schema registry provides an on chain repository for schemas, thereby allowing participants of the network to flexibly interact and exchange messages with each other without facing the challenge of sharing, managing and validating messages as well as schemas between them.

Using schema registry, message producers no longer need to include full schema with the message payload, instead only include ID of that schema, resulting in efficient serialization.

![registry](https://user-images.githubusercontent.com/61435908/163263866-adf36d23-0968-42cd-8d50-6025bb7c455b.png)

### Schema Storage

- **Type definition**: ```StorageMap<_, Twox64Concat, SchemaId, BoundedVec<u8,T::MaxSchemaSize>>```
- **Description**: Schemas are stored as key-value pair of SchemaId vs Serialized schema payload allowed to a maximum size.
- **Implementation**: MRC will expose a substrate extrinsic ``` register_schema ``` to allow participants store a schema on chain.

### Schema Validation

Schema registry performs following checks before onboarding a schema on MRC:
    - Ensure payload is signed by an authorizing AccountId
    - Ensure MRC  
