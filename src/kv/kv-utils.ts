import { KvMetadata } from '@/kv/kv-models.ts';

export function parseMetadataJSON(value: string): KvMetadata {
  if (value === '') {
    return null;
  }

  try {
    return JSON.parse(value);
  } catch (e) {
    console.error('Error parsing JSON:', e);
    return null;
  }
}

export function validateMetadata(value: string): boolean {
  if (value === '') {
    return true;
  }

  try {
    JSON.parse(value);
    return true;
  } catch (e) {
    return false;
  }
}
