import { OrderDirection } from '@/lib/common-models.ts';

export interface BucketsDTO {
  items: BucketDTO[];
  page_info?: CursorPageInfo;
}

export interface CursorPageInfo {
  cursor: string;
  count?: number;
  per_page?: number;
}

export interface BucketDTO {
  name: string;
  creation_date: string;
  jurisdiction?: BucketJurisdiction;
  location?: BucketLocation;
  storage_class?: BucketStorageClass;
}

export interface Bucket {
  name: string;
  creation_date: Date;
  jurisdiction?: BucketJurisdiction;
  location?: BucketLocation;
  storage_class?: BucketStorageClass;
}

export enum BucketJurisdiction {
  DEFAULT = 'default',
  EU = 'eu',
  FED_RAMP = 'fedramp',
}

export enum BucketLocation {
  APAC = 'apac',
  EEUR = 'eeur',
  ENAM = 'enam',
  WEUR = 'weur',
  WNAM = 'wnam',
  OC = 'oc',
}

export enum BucketStorageClass {
  STANDARD = 'Standard',
  INFREQUENT_ACCESS = 'InfrequentAccess',
}

export interface BucketsListInput {
  account_id: string;
  cursor?: string;
  direction?: OrderDirection;
  name_contains?: string;
  order?: BucketOrder;
  per_page?: number;
  start_after?: string;
  jurisdiction?: BucketJurisdiction;
}

export enum BucketOrder {
  NAME = 'name',
}
