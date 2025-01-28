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

export interface AccountWithToken {
  id: string;
  name: string;
  token: Token;
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
