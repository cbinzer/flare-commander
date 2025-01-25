export class AuthenticationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AuthenticationError';
  }
}

export class InvalidTokenError extends AuthenticationError {
  constructor(message = 'Invalid token') {
    super(message);
    this.name = 'InvalidTokenError';
  }
}

export class ExpiredTokenError extends AuthenticationError {
  constructor(message = 'Expired token') {
    super(message);
    this.name = 'ExpiredTokenError';
  }
}

export class DisabledTokenError extends AuthenticationError {
  constructor(message = 'Disabled token') {
    super(message);
    this.name = 'DisabledTokenError';
  }
}

export class InvalidAccountIdError extends AuthenticationError {
  constructor(message = 'Invalid account ID') {
    super(message);
    this.name = 'InvalidAccountIdError';
  }
}

export class UnknownAuthenticationError extends AuthenticationError {
  constructor(message: string) {
    super(message);
    this.name = 'UnknownAuthenticationError';
  }
}
