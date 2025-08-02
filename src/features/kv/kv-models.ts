import * as zod from 'zod';

export interface KvNamespaces {
  items: KvNamespace[];
  page_info: PageInfo;
}

export interface KvNamespacesListInput {
  account_id: string;
  order_by?: KvNamespacesOrderBy;
  order_direction?: OrderDirection;
  page?: number;
  per_page?: number;
}

export enum KvNamespacesOrderBy {
  ID = 'id',
  TITLE = 'title',
}

export enum OrderDirection {
  ASC = 'asc',
  DESC = 'desc',
}

export interface KvNamespace {
  id: string;
  title: string;
  beta?: boolean;
  supports_url_encoding?: boolean;
}

export interface KvNamespaceGetInput {
  account_id: string;
  namespace_id: string;
}

export interface KvKeysListInput {
  account_id: string;
  namespace_id: string;
  cursor?: string;
  limit?: number;
  prefix?: string;
}

export interface PageInfo {
  count: number;
  page: number;
  per_page: number;
  total_count: number;
}

export interface KvNamespaceCreateInput {
  account_id: string;
  title: string;
}

export interface KvNamespaceUpdateInput extends KvNamespaceCreateInput {
  namespace_id: string;
}

export interface KvNamespaceDeleteInput {
  account_id: string;
  namespace_id: string;
}

export interface KvPair {
  key: string;
  value?: Uint8Array;
  expiration?: Date;
  metadata?: KvMetadata;
}

export const kvPairsImportParser = zod.array(
  zod.object({
    key: zod.string(),
    value: zod.optional(zod.array(zod.uint32())),
    expiration: zod.optional(zod.number().nonnegative()),
    metadata: zod.optional(zod.json()),
  }),
);

export interface KvPairGetInput {
  account_id: string;
  namespace_id: string;
  key: string;
}

export interface KvPairsGetInput {
  account_id: string;
  namespace_id: string;
  keys: string[];
}

export interface KvPairDTO {
  key: string;
  value?: number[];
  expiration?: number;
  metadata?: KvMetadata;
}

export interface KvPairCreateInput {
  account_id: string;
  namespace_id: string;
  key: string;
  value?: Uint8Array;
  expiration?: Date;
  expiration_ttl?: number;
  metadata?: KvMetadata;
}

export interface KvPairWriteInput {
  account_id: string;
  namespace_id: string;
  key: string;
  value?: Uint8Array;
  expiration?: number;
  expiration_ttl?: number;
  metadata?: KvMetadata;
}

export interface KvPairsWriteInput {
  account_id: string;
  namespace_id: string;
  pairs: KvPairsWriteInputPair[];
}

export type KvPairsWriteInputPair = Omit<KvPairWriteInput, 'account_id' | 'namespace_id'> & { value: number[] };

export interface KvPairsWriteResult {
  successful_key_count: number;
  unsuccessful_keys: string[];
}

export interface KvPairsDeleteInput {
  account_id: string;
  namespace_id: string;
  keys: string[];
}

export interface KvPairsDeleteResult {
  successful_key_count: number;
  unsuccessful_keys: string[];
}

export interface KvKeys {
  keys: KvKey[];
  cursor?: string;
}

export interface KvKeysDTO {
  keys: KvKeyDTO[];
  cursor?: string;
}

export type KvMetadata = string | number | boolean | null | Record<string, unknown> | Array<unknown>;

export interface KvKey {
  name: string;
  expiration?: Date;
  metadata?: KvMetadata;
}

export interface KvKeyDTO {
  name: string;
  expiration?: number;
  metadata?: KvMetadata;
}

export interface KvTableKey extends KvKey {
  namespaceId: string;
}

export type KvErrorKind =
  | 'Authentication'
  | 'Unknown'
  | 'NamespaceAlreadyExists'
  | 'KeyAlreadyExists'
  | 'InvalidMetadata'
  | 'InvalidExpiration';

export class KvError extends Error {
  constructor(
    message: string,
    public kind: KvErrorKind,
  ) {
    super(message);
    this.name = 'KvError';
  }
}
