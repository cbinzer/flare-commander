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
