import { CredentialsType, UserAuthTokenCredentials } from '@/features/authentication/auth-models.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import {
  KvError,
  KvKeyPairWriteInput,
  KvPair,
  KvPairCreateInput,
  KvPairDTO,
  KvPairGetInput,
} from '@/features/kv/kv-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { convertPlainToKvErrorClass } from '@/features/kv/lib/kv-utils.ts';

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
    setError(null);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const pair = await invokeGetKvPair(
        { account_id: account?.id ?? '', namespace_id: namespaceId, key },
        credentials,
      );
      setKvPair(pair);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoading(false);
    }
  };

  const createKvPair = async (input: Omit<KvPairCreateInput, 'account_id'>) => {
    setIsCreating(true);
    setError(null);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
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
    setError(null);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
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
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

export async function invokeCreateKvPair(
  input: KvPairCreateInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPair> {
  try {
    console.log(input);
    const kvPair = await invoke<KvPairDTO>('create_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvPair,
      expiration: kvPair.expiration ? new Date(kvPair.expiration * 1000) : undefined,
    };
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
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
    throw convertPlainToKvErrorClass(e as KvError);
  }
}
