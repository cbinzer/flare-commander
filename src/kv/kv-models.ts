export interface KvNamespace {
  id: string;
  title: string;
}

export interface KvItems {
  items: KvItem[];
  cursor?: string;
}

export interface KvItem {
  key: string;
  value: string;
  expiration?: Date;
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
