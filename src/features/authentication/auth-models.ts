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
  credentials: Credentials;
}

export type Credentials =
  | UserAuthKeyCredentials
  | UserAuthTokenCredentials
  | AccountAuthTokenCredentials
  | ServiceCredentials;

export interface CredentialsBase {
  type: CredentialsType;
}

export enum CredentialsType {
  UserAuthKey = 'UserAuthKey',
  UserAuthToken = 'UserAuthToken',
  AccountAuthToken = 'AccountAuthToken',
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

export interface AccountAuthTokenCredentials extends CredentialsBase {
  type: CredentialsType.AccountAuthToken;
  token: string;
}

export interface ServiceCredentials extends CredentialsBase {
  type: CredentialsType.Service;
  key: string;
}
