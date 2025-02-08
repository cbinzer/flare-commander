export type AuthenticationErrorKind =
  | 'InvalidToken'
  | 'ExpiredToken'
  | 'DisabledToken'
  | 'InvalidAccountId'
  | 'Unknown';

export class AuthenticationError extends Error {
  constructor(
    message: string,
    public kind: AuthenticationErrorKind,
  ) {
    super(message);
    this.name = 'AuthenticationError';
  }
}

export interface AccountWithCredentials {
  id: string;
  name: string;
  credentials: AccountCredentials;
}

export type Credentials =
  | UserAuthKeyCredentials
  | UserAuthTokenCredentials
  | ServiceCredentials;

export interface CredentialsBase {
  type: CredentialsType;
  account_id: string;
}

export enum CredentialsType {
  UserAuthKey = 'UserAuthKey',
  UserAuthToken = 'UserAuthToken',
  Service = 'Service',
}

export interface UserAuthKeyCredentials extends CredentialsBase {
  type: CredentialsType.UserAuthKey;
  email: string;
  key: string;
}

export interface UserAuthTokenCredentials extends CredentialsBase {
  type: CredentialsType.UserAuthToken;
  token: string;
}

export interface ServiceCredentials extends CredentialsBase {
  type: CredentialsType.Service;
  key: string;
}

export type AccountCredentials =
  | AccountUserAuthKeyCredentials
  | AccountUserAuthTokenCredentials
  | AccountServiceCredentials;

export interface AccountUserAuthKeyCredentials {
  type: CredentialsType.UserAuthKey;
  email: string;
  key: string;
}

export interface AccountUserAuthTokenCredentials {
  type: CredentialsType.UserAuthToken;
  token: string;
}

export interface AccountServiceCredentials {
  type: CredentialsType.Service;
  key: string;
}

export interface Account {
  id: string;
  name: string;
}

export interface Account {
  id: string;
  name: string;
}

export interface Token {
  id: string;
  value?: string;
  status: TokenStatus;
  policies?: TokenPolicy[];
}

export enum TokenStatus {
  ACTIVE = 'active',
  DISABLED = 'disabled',
  EXPIRED = 'expired',
}

export interface TokenPolicy {
  id: string;
  effect: TokenPolicyEffect;
  permission_groups: PermissionGroup[];
  resources: Record<string, string>;
}

export enum TokenPolicyEffect {
  ALLOW = 'allow',
  DENY = 'deny',
}

export interface PermissionGroup {
  id: String;
  meta?: PermissionGroupMeta;
  name?: string;
}

export interface PermissionGroupMeta {
  key?: string;
  value?: string;
}
