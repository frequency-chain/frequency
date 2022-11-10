export const PARQUET_BROADCAST = [
    {
        name: "announcementType",
        column_type: {
        INTEGER: {
            bit_width: 32,
            sign: true,
        },
        },
        compression: "GZIP",
        bloom_filter: false,
    },
    {
        name: "contentHash",
        column_type: "BYTE_ARRAY",
        compression: "GZIP",
        bloom_filter: true,
    },
    {
        name: "fromId",
        column_type: {
        INTEGER: {
            bit_width: 64,
            sign: false,
        },
        },
        compression: "GZIP",
        bloom_filter: true,
    },
    {
        name: "url",
        column_type: "STRING",
        compression: "GZIP",
        bloom_filter: false,
    },
];

export const AVRO_GRAPH_CHANGE = {
    type: "record",
    name: "GraphChange",
    fields: [
      // When converting from Frequency Schema Message to DSNP Announcement, assume announcementType=1
      {
        name: "changeType",
        type: {
          name: "ChangeTypeEnum",
          type: "enum",
          symbols: ["Unfollow", "Follow"], // Encoded as int
        },
      },
      {
        name: "fromId",
        type: {
          name: "DSNPId",
          type: "fixed",
          size: 8,
        },
      },
      {
        name: "objectId",
        type: "DSNPId",
      },
    ],
  };