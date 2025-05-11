export interface KvNamespaces {
  items: KvNamespace[];
  page_info: PageInfo;
}

export interface KvNamespacesListInput {
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

export interface PageInfo {
  count: number;
  page: number;
  per_page: number;
  total_count: number;
}

export interface KvNamespaceCreateInput {
  title: string;
}

export interface KvNamespaceUpdateInput extends KvNamespaceCreateInput {
  id: string;
}

export interface KvItem {
  key: string;
  value?: string;
  expiration?: Date;
  metadata?: KvMetadata;
}

export interface KvItemDTO {
  key: string;
  value?: string;
  expiration?: number;
  metadata?: KvMetadata;
}

export interface KvKeyPairCreateInput {
  namespaceId: string;
  key: string;
  value?: string;
  expiration?: Date;
  expiration_ttl?: number;
  metadata?: KvMetadata;
}

export interface KvKeyPairUpsertInput {
  namespaceId: string;
  key: string;
  value?: string;
  expiration?: Date;
  expiration_ttl?: number;
  metadata?: KvMetadata;
}

export interface KvItemsDeletionInput {
  namespace_id: string;
  keys: string[];
}

export interface KvItemsDeletionResult {
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

export type KvErrorKind = 'Authentication' | 'Unknown' | 'NamespaceAlreadyExists' | 'KeyAlreadyExists';

export class KvError extends Error {
  constructor(
    message: string,
    public kind: KvErrorKind,
  ) {
    super(message);
    this.name = 'KvError';
  }
}
