# Summary

In other documents, we explore ways to structure an explicit batch of messages.
That is, in terms of concrete structure. This document has a different aim --
deriving the idea of a batch logically, as a product of fields present on other
entities in our system.

## Background

We are currently experimenting with a new system architecture that has the
following entities:

1. Schema
   - Id
   - Format
   - FormatType
   - PayloadLocation
2. Message
   - Off-chain / On-chain Payload

*Protocols* would be responsible for indicating not only the shape of the
underlying payload, but also its format and location. This is important, because
a protocol's format property would indicate whether a given payload contains a
batch or not.

## FormatType

Payloads can either be of format type `PARQUET` or `AVRO`. These file formats
indicate specifically whether their payloads are singular or arrays of objects
(More info needed here).

## PayloadLocation

The combination of format and location constrain possible payload types:

```txt
| Format  | Location        | Payload           |
-------------------------------------------------
| Avro   | On-chain         | Public Graph      |
| Parquet| On-chain         | ??                |
| Avro   | IPFS (Off-chain) | Private Graph (?) |
| Parquet| IPFS (Off-chain) | Announcement      |
```

(More needed here).

## Message Payloads

Message payloads can be either on-chain or off-chain. If they are off-chain,
there is a good chance that they will be located on IPFS.

## Batch as a Logical Construct

We can circumvent defining a batch explicitly if we leverage the file format
included in the protocol. Since both Avro and Parquet file types declare whether
or not they contain single or plural values, the format itself could indicate
whether or not the message is a batch.

## Rationale

There are some upsides to deriving batching logically from existing structures.
One is costsavings. Not having a batch structure means we don't need to worry
about any on-chain computation associated with batch messages -- we simply look
at the format and location on the parent schema and we can deduce whether the
file is singular or plural.

(More needed).
