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
| Operation  | Description |
| ------------- | ------------- |
| **Initialise**  | Creates in memory graph structure for a desired MSA.  |
| **Import**  | Import the blob from frequency into Graph SDK for desired MSA.  |
| **Update**  | Updates the current in-memory graph structure with incoming updates.  |
| **Get Graph**  | Exposes current state of in-memory graph to the consumer of sdk.  |
| **Has Updates**  | Determines if the applied changes created some updates in graph that needs to be persisted on Frequency.  |
| **Calculate Updates**  | Applies the graph changes and generates optimized blobs to be applied to Frequency  |
| **Persist**  | This should be called after successful update of Frequency to remove tracking of persisted updates.  |

#### Import actions
|  |  sub action |
| ------------- | ------------- |
| 1 | Deserialize the blob to specified schema. |
| 2 | Decrypt encrypted fields using DSNP version specified algorithm. (if any). |
| 3 | Decompress compressed fields using DSNP version specified algorithm. (if any). |
| 4 | Verify plain data and PRI ids. (if any) |
| 5 | Add verified plain data into in-memory graph data structure. |

#### Update related types
|  |  Types |
| ------------- | ------------- |
| Update types |     - Add <br /> - Remove |
| Privacy levels |     - Public <br /> - Private |
| Relationship types |     - Follow <br /> - Friendship |

## Interface

* ###### import
  * params
    * _**graph_data**_: A list of `Import` type

* ###### getConnections
  * params
    * _**msa_id**_: Owner of the graph we are trying to read/write
    * _**privacy_level**_: Public or Private
    * _**relationship_type**_: Follow or Friendship
  * returns
    *  A list of `Connection` type

* ###### SetPublicKeys
  * params
    * _**msa_keys**_: A list of msa ids and their public keys to be able to calculate PRI ids of
`MsaKey` type

* ###### ApplyActions
  * _**actions**_: a list of connect or disconnect actions of `Action` type

* ###### CalculateUpdates
    * returns
        * A list of `Update` type

* ###### RotateKeys
  * params:
    * a list of `Rotation` type

## Concepts and entities

```rust
type Page = Vec<u8>;

pub enum PrivacyType {
    Public,
    Private,
}

pub enum ConnectionType {
    Follow,
    Friendship,
}

pub struct Import {
    pub msa_id: MessageSourceId,
    pub keys: Vec<KeyPair>,    // need to define KeyPair based on NaCl library
    pub pages: Vec<Page>,
}

pub struct Connection {
    pub msa_id: MessageSourceId,
    pub privacy_type: PrivacyType,
    pub connection_type: ConnectionType,
}

pub struct MsaKey {
    pub msa_id: MessageSourceId,
    pub keys: Vec<PublicKey>,  // need to define PublicKey based on NaCl library
}

pub enum Action {
    Connect {
        owner_msa_id: MessageSourceId,
        connection: Connection,
        connection_key: Option<PublicKey>,
    },
    Disconnect {
        owner_msa_id: MessageSourceId,
        connection: Connection,
    },
}

pub enum Update {
    Persist {
        owner_msa_id: MessageSourceId,
        schema_id: SchemaId,
        page_id: PageId,
        prev_hash: Vec<u8>,
        payload: Vec<u8>,
    },
    Delete {
        owner_msa_id: MessageSourceId,
        schema_id: SchemaId,
        page_id: PageId,
        prev_hash: Vec<u8>,
    },
}

pub struct Rotation {
    owner_msa_id: MessageSourceId,
    prev_keys: Vec<KeyPair>,
    new_key: KeyPair,
}
```

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
