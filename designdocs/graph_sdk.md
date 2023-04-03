# Graph SDK

## Context and Scope

The proposed design consists of changes that is going to be a separate [repository](https://github.com/LibertyDSNP/graph-sdk) to facilitate
graph related interactions with Frequency chain in test and production environments.

## Problem Statement

Implementing DSNP protocol on Frequency comes with its own challenges and tradeoffs. One of these
challenges is to decide what is the boundary between DSNP and Frequency and what should be
implemented directly in Frequency.

To keep Frequency implementation as DSNP agnostic as possible we decided to store social graph
related data as blobs in Frequency and keep the details of how these blobs are created and modified
inside DSNP spec. Graph SDK is an implementation of mentioned DSNP spec for social graph, optimized
for storage and interaction with Frequency.
## Goals

- Define operations
- Define interface of Graph SDK.
- Define the main entities in Graph SDK.
- Define the memory model of the graph.
- Define the algorithms to optimize the output regarding Frequency.

## Operations
Following is a list of desired operations in this SDK:

| Name  | Description |
| ------------- | ------------- |
| **Initialise**  | Creates in memory graph structure for a desired DSNP user.  |
| **Import**  | Import the blob(s) and keys from frequency into Graph SDK for desired DSNP user.  |
| **Update**  | Changes the current in-memory graph structure with incoming updates.  |
| **Get Graph**  | Exposes current state of in-memory graph to the consumer of the SDK.  |
| **Has Updates**  | Determines if the applied changes created some updates in graph that needs to be persisted on Frequency.  |
| **Calculate Updates**  | Applies the graph changes and generates optimized blobs to be applied to Frequency  |

#### Import actions
Steps necessary to import a social graph blob:

|  |  sub action | Condition |
| ------------- | ------------- | ------------- |
| 1 | Deserialize the blob to specified schema. | Always |
| 2 | Decrypt encrypted fields using DSNP version specified algorithm. | If encrypted |
| 3 | Decompress compressed fields using DSNP version specified algorithm. | If Compressed |
| 4 | Verify plain data and PRI ids | If private friendship |
| 5 | Add verified plain data into in-memory graph data structure. | Always |

#### Update related types
| Name |  Types |
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
    * _**dsnp_user_id**_: Owner of the graph we are trying to read/write
    * _**privacy_level**_: Public or Private
    * _**relationship_type**_: Follow or Friendship
    * _**include_local_changes**_: Include local changes in graph or just return the stable instance
  * returns
    * A list of `Connection` type

* ###### SetPublicKeys
  * usage
    * This function allows calculating PRIds for Private Friendship connections
  * params
    * _**dsnp_keys**_: A list of DSNP user ids and their public keys to be able to calculate PRI ids of
`DsnpKey` type

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
    pub dsnp_user_id: MessageSourceId,
    pub keys: Vec<KeyPair>,    // need to define KeyPair based on NaCl library
    pub pages: Vec<Page>,
}

pub struct Connection {
    pub dsnp_user_id: MessageSourceId,
    pub privacy_type: PrivacyType,
    pub connection_type: ConnectionType,
}

pub struct DsnpKey {
    pub dsnp_user_id: MessageSourceId,
    pub keys: Vec<PublicKey>,  // need to define PublicKey based on NaCl library
}

pub enum Action {
    Connect {
        owner_dsnp_user_id: MessageSourceId,
        connection: Connection,
        connection_key: Option<PublicKey>, // included only if PRId calculation is required
    },
    Disconnect {
        owner_dsnp_user_id: MessageSourceId,
        connection: Connection,
    },
}

pub enum Update {
    Persist {
        owner_dsnp_user_id: MessageSourceId,
        schema_id: SchemaId,
        page_id: PageId,
        prev_hash: Vec<u8>,
        payload: Vec<u8>,
    },
    Delete {
        owner_dsnp_user_id: MessageSourceId,
        schema_id: SchemaId,
        page_id: PageId,
        prev_hash: Vec<u8>,
    },
}

pub struct Rotation {
    owner_dsnp_user_id: MessageSourceId,
    prev_key: KeyPair,
    new_key: KeyPair,
}
```

![Entities](https://user-images.githubusercontent.com/9152501/222261121-185d4d9d-1ecb-4ffa-8fe0-8612e58d7b27.png)

* **Tracker**: This is used only to allow generating optimized pages for Frequency.

## Memory model and usage
It is recommended to batch the graph changes for DSNP users as much as possible and initialize the
library with all the keys and page blobs related to desired users. Apply all changes to in-memory
graph and calculate and apply all page updates to Frequency chain.

To ensure local graph state is in sync with the chain graph state, it is recommended to
only initialize and use the library in case there are any changes to the graph, instead of having a
long living in-memory instance of the graph. This would minimize the probability of having local
state being stale.

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

#### Fullness check
To relax the optimization computation on pages we can define a threshold of fullness along with hard
cutoffs. If the newly added data causes this page to pass the defined threshold we will consider
that the page became full after addition.
Thresholds are helpful to reduce the possibility of adding a new data passing the hard cutoff point
in which requires the data to be calculated twice. First to ensure not passing the cutoff point and
then to actually add the data to the page.
