# Graph SDK

## Context and Scope

The proposed design consists of changes that is going to be a separate repository to facilitate
graph related interactions with Frequency chain in test and production environments

## Problem Statement

Implementing DSNP protocol on Frequency comes with its own challenges and tradeoffs. One of these
challenges is to decide what is the boundary between DSNP and Frequency and what should be
implemented directly in Frequency.

To keep Frequency implementation as DSNP agnostic as possible we decided to store social graph
related data as blobs in Frequency and keep the details of how these blobs are created and modified
inside DSNP spec. Graph SDK is an implementation of mentioned DSNP spec for social graph optimized
for storage and interaction with Frequency.
## Goals

- Define operations
- Define the interface for Graph SDK.
- Define the main concepts and entities in Graph SDK.
- Define the algorithms to optimize the output regarding Frequency.

## Operations
* **Initialise** : Creates in memory graph structure for a desired MSA.
* **Import** : Import the blob from frequency into Graph SDK for desired MSA. Successful import is
consists of following actions.
  * Deserialize the blob to specified schema.
  * Decrypt encrypted fields using DSNP version specified algorithm. (if any)
  * Decompress compressed fields using DSNP version specified algorithm. (if any)
  * Verify plain data and PRI ids. (if any)
  * Add verified plain data into in-memory graph data structure.
* **Update** : Updates the current in-memory graph structure with incoming updates. Each update has
the following details.
  * Update types:
    * Add
    * Remove
  * Privacy levels:
    * Public
    * Private
  * Relationship types:
    * Follow
    * Friendship
* **Get Graph** : Exposes current state of in-memory graph to the consumer of sdk.
* **Has Updates** : Determines if the applied changes created some updates in graph that needs to be
persisted on Frequency.
* **Calculate Updates** : Applies the graph changes and generates optimized blobs to be applied to
frequency. These updates will have the following details
  * MSA
  * Location: the details of schema and page number. (if any)
  * Graph data blob
* **Persist** : This should be called after successful update of Frequency to remove tracking of
persisted updates.

## Interface

TODO: add interface

## Concepts and entities

![Entities](https://user-images.githubusercontent.com/9152501/222261121-185d4d9d-1ecb-4ffa-8fe0-8612e58d7b27.png)

* **Tracker**: This is used only to allow generating optimized pages for Frequency.

## Algorithms

We would like to minimize the number of transactions and related data which needs to be submitted to
Frequency.

An example of such an algorithm would be as follows
1. apply all removes before adds to determine all pages that are required to be changed
2. apply adds to pages that are required to be changed and check the page size
3. if page size is bigger than supported try the next changing page
4. if there are no changing pages start with existing pages with the least data (or last page)
and do check in step 3.
5. if there are no pages like that create a new page
