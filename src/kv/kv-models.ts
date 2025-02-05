export interface KvNamespace {
  id: string;
  title: string;
}

export type KvErrorKind = 'Authentication' | 'Unknown';

export class KvError extends Error {
  constructor(
    message: string,
    public kind: KvErrorKind,
  ) {
    super(message);
    this.name = 'KvError';
  }
}
