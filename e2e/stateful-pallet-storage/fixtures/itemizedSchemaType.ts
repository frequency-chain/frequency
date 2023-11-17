export const AVRO_CHAT_MESSAGE = {
  type: 'record',
  name: 'ChatMessage',
  fields: [
    {
      name: 'fromId',
      type: {
        name: 'DSNPId',
        type: 'fixed',
        size: 8,
      },
    },
    {
      name: 'message',
      type: 'string',
    },
    {
      name: 'inReplyTo',
      type: ['null', 'DSNPId'],
      default: null,
    },
    {
      name: 'url',
      type: ['null', 'string'],
      default: null,
    },
  ],
};
