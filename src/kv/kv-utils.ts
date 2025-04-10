import { KvMetadata } from '@/kv/kv-models.ts';

export function stringifyMetadataJSON(value: KvMetadata): string {
  if (value === null) {
    return '';
  }

  try {
    return JSON.stringify(value);
  } catch (e) {
    console.error('Error stringifying JSON:', e);
    return '';
  }
}

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
