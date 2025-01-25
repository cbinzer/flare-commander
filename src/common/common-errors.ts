export interface APIError {
  kind:
    | 'InvalidToken'
    | 'ExpiredToken'
    | 'DisabledToken'
    | 'InvalidAccountId'
    | 'Unknown';
  message: string;
}
