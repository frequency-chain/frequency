# MRC Message Schemas

## Context and Scope

Messages on MRC are validated and stored against pre-defined schema(s). In order to support a variety of message types, it is imperative to define an on-chain semantics, pallet(s) for example, to handle dynamic registration, validation, storage and retention schemes for schemas.
This document describes how schemas are handled on chain in the following sections.

## Problem Statement

Message passing is a core functionality to social networks. The way to enforce a communication protocol between participants of social network via services is done by messaging schema. Analogous to interfaces, schemas provide a strongly typed description for messages ensuring their correctness, validity, extensibility and interoperability between services interacting with MRC.

## Goals

At a minimum, MRC should implement procedures to register, validate, store and access variety of messaging schemas dynamically. Schemas on chain must have the following salient features:

- **Registry** : Implement a schema registry, enabling participants to register and store validated schemas on chain.
- **Validation** : Implement procedural calls enabling consumers to validate messages against stored schema. Due to serialization concerns schema and message validation will be done off chain.
- **Interfaces** : Implement appropriate procedural calls to perform CRUD operations on schema registry.
- **Retention** : Implement some sort of schema(s) retention logic  for optimal on-chain storage. Retention periods per schema can be modified via super user permissions.

## Proposal

This document outlines various components of MRC schemas, including, but not limited to, ability of network participants to register, validate and access message schemas dynamically via on chain semantics.

### Schema Registry

Schema registry provides an on chain repository for schemas, thereby allowing participants of the network to flexibly interact and exchange messages with each other without facing the challenge of sharing, managing and validating messages as well as schemas between them.

Using schema registry, message producers no longer need to include full schema with the message payload, instead only include ID of that schema, resulting in efficient serialization.

```mermaid
  graph TD;
      A-->B;
      A-->C;
      B-->D;
      C-->D;
```
