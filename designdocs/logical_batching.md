# Summary

In other documents, we explore ways to structure an explicit batch of messages.
That is, in terms of concrete structure. This document has a different aim --
deriving the idea of a batch logically, as a product of fields present on other
entities in our system.

## Background

We are currently experimenting with a new system architecture that has the
following entities:

1. Schema
   - id: u16
   - format: Format
   - format_type: FormatType
   - payload_location: PayloadLocation
2. Message
   - payload: Payload (TBD)
   - payload_size: usize

*Schemas* would be responsible for indicating not only the shape of the
underlying payload, but also its format and location. This is important, because
a schema's format property would indicate whether a given payload contains a
batch or not.

## Id

```rust
type Id = u16
```

This field serves as a unique identifier for the schema.

## Format

```rust
struct Format {
  message_schema_id: u16,
  file_size: usize,
}
```

The Schema's format refers to the shape of the underlying message payload.

## FormatType

```rust
enum FormatType {
  PARQUET,
  AVRO
}
```

Payloads can either be of format type `PARQUET` or `AVRO`. These file formats
indicate specifically whether their payloads are singular or arrays of objects
(More info needed here).

## PayloadLocation

```rust
enum PayloadLocation {
  OnChain,
  IPFSCIDv2
}
```

The combination of format and location constrain possible payload types:

```txt
| Format  | Location        | Payload                               |
---------------------------------------------------------------------
| Avro   | On-chain         | DSNP Graph Change                     |
| Parquet| On-chain         | Unknown                               |
| Avro   | IPFS (Off-chain) | Larger Avro structures                |
| Parquet| IPFS (Off-chain) | DSNP Broadcast or Reply Announcements |
```

(More needed here).

## Message Payloads

Message payloads can be either on-chain or off-chain. If they are off-chain,
they will be stored on IPFS.

## Batch as a Logical Construct

We can circumvent defining a batch explicitly if we leverage the file format
included in the schema. Since both Avro and Parquet file types declare whether
or not they contain single or plural values, the format itself could indicate
whether or not the message is a batch.

## Rationale

There are some upsides to deriving batching logically from existing structures.
One is cost savings. Not having a batch structure means we don't need to worry
about any on-chain computation associated with batch messages -- we simply look
at the format and location on the parent schema and we can deduce whether the
file is singular or plural.

(More needed).
