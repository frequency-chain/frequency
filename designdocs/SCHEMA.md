# MRC Message Schemas

## Context and Scope

Messages on MRC are validated and stored against pre-defined schema(s). In order to support a variety of message types, it is imperative to define an on-chain semantics, pallet(s) for example, to handle dynamic registration, validation, storage and retention schemes for schemas.
This document describes how schemas are handled on chain in the following sections.

## Problem Statement

Message passing is a core functionality to social networks. The way to enforce a communication protocol between participants of social network via services is done by messaging schema. Analogous to interfaces, schemas provide a strongly typed description for messages ensuring their correctness, validity, extensibility and interoperability between services interacting with MRC.

## Goals

At a minimum, MRC should implement procedures to register, validate, store and access variety of messaging schemas dynamically. Schemas on chain must have the following salient features:

- Implement a schema registry, enabling participants to register and store validated schemas.
- Implement appropriate procedural calls to perform CRUD operations on schema registry.
- Implement some retention logic for schemas for optimal on-chain storage.
