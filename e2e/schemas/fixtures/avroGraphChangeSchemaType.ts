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