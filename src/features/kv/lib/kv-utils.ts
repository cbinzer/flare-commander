import { KvError, KvMetadata } from '@/features/kv/kv-models.ts';

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

export function validateExpirationTTL(value: string): boolean {
  const expirationTTL = Number(value);
  console.log(expirationTTL);
  if (isNaN(expirationTTL)) {
    return false;
  }

  return expirationTTL === 0 || expirationTTL >= 60;
}

export function convertPlainToKvErrorClass(kvError: KvError): KvError {
  console.error(kvError);
  return new KvError(kvError.message, kvError.kind);
}
