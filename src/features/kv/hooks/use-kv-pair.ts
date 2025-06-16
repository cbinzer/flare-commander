import { CredentialsType, UserAuthTokenCredentials } from '@/features/authentication/auth-models.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import {
  KvError,
  KvPairDTO,
  KvKeyPairWriteInput,
  KvPair,
  KvPairCreateInput,
  KvPairGetInput,
} from '@/features/kv/kv-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

export function useKvPair() {
  const { account } = useAuth();
  const [kvPair, setKvPair] = useState<KvPair | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isWriting, setIsWriting] = useState(false);
  const [error, setError] = useState<KvError | null>(null);

  const getKvPair = async (namespaceId: string, key: string) => {
    setIsLoading(true);
    setKvPair(null);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const item = await invokeGetKvPair(
        { account_id: account?.id ?? '', namespace_id: namespaceId, key },
        credentials,
      );
      setKvPair(item);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoading(false);
    }
  };

  const createKvPair = async (input: Omit<KvPairCreateInput, 'account_id'>) => {
    setIsCreating(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };
    try {
      const createdKvPair = await invokeCreateKvPair({ ...input, account_id: account?.id ?? '' }, credentials);
      setKvPair(createdKvPair);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsCreating(false);
    }
  };

  const writeKvPair = async (input: Omit<KvKeyPairWriteInput, 'account_id'>) => {
    setIsWriting(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const updatedKvPair = await invokeWriteKvPair({ ...input, account_id: account?.id ?? '' }, credentials);
      setKvPair(updatedKvPair);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsWriting(false);
    }
  };

  return {
    kvPair,
    getKvPair,
    createKvPair,
    writeKvPair,
    isLoading,
    isCreating,
    isWriting,
    error,
  };
}

export async function invokeGetKvPair(input: KvPairGetInput, credentials: UserAuthTokenCredentials): Promise<KvPair> {
  try {
    const kvPair = await invoke<KvPairDTO>('get_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvPair,
      expiration: kvPair.expiration ? new Date(kvPair.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeCreateKvPair(
  input: KvPairCreateInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPair> {
  try {
    const kvPair = await invoke<KvPairDTO>('create_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvPair,
      expiration: kvPair.expiration ? new Date(kvPair.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeWriteKvPair(
  input: KvKeyPairWriteInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPair> {
  try {
    const kvPair = await invoke<KvPairDTO>('write_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvPair,
      expiration: kvPair.expiration ? new Date(kvPair.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}
