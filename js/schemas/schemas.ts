import { SCHEMA_INFOS } from './data';

export interface SchemaInfo {
  id: number;
  name: string;
  namespace: string;
  version: number;
  modelType: string;
  payloadLocation: string;
  appendOnly: boolean;
  signatureRequired: boolean;
  deprecated: boolean;
}

const schemaFullName = (info: SchemaInfo): string => `${info.namespace}.${info.name}@v${info.version}`;

/**
 * Mapping that will allow us to get schema full names from their ids
 */
export const ID_TO_SCHEMA_FULL_NAME = new Map<number, string>(SCHEMA_INFOS.map((x) => [x.id, schemaFullName(x)]));

/**
 * Mapping that will allow us to get schema ids from their full names
 */
export const FULL_NAME_TO_ID = new Map<string, number>(SCHEMA_INFOS.map((x) => [schemaFullName(x), x.id]));

/**
 * Mapping that will allow us to get schema infos from their IDs
 */
export const ID_TO_SCHEMA_INFO = new Map<number, SchemaInfo>(SCHEMA_INFOS.map((x) => [x.id, x]));
