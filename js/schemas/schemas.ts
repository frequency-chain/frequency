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
const schemaFullNameWithoutVersion = (info: SchemaInfo): string => `${info.namespace}.${info.name}`;

/**
 * Mapping that will allow us to get schema full names from their ids
 */
export const ID_TO_SCHEMA_FULL_NAME = new Map<number, string>(SCHEMA_INFOS.map((x) => [x.id, schemaFullName(x)]));

/**
 * Mapping that will allow us to get schema ids from their full names
 */
export const FULL_NAME_TO_ID = new Map<string, number>(SCHEMA_INFOS.map((x) => [schemaFullName(x), x.id]));

/**
 * Mapping that will allow us to get active schema ids from their names
 * example input dsnp.public-key-key-agreement
 */
export const NAME_TO_ID_ACTIVE = new Map<string, number>(
  SCHEMA_INFOS.filter((info) => !info.deprecated).map((x) => [schemaFullNameWithoutVersion(x), x.id])
);

/**
 * Mapping that will allow us to get schema infos from their IDs
 */
export const ID_TO_SCHEMA_INFO = new Map<number, SchemaInfo>(SCHEMA_INFOS.map((x) => [x.id, x]));

/**
 * A method that can retrieve all versions of a given schema name without any version
 * example input dsnp.public-key-key-agreement
 */
export const getAllVersionsFromSchemaName = (schemaName: string): SchemaInfo[] => {
  return SCHEMA_INFOS.filter((info) => schemaFullNameWithoutVersion(info) === schemaName);
};
