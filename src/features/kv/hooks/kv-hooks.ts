import { CredentialsType, UserAuthTokenCredentials } from '@/features/authentication/auth-models.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import {
  KvError,
  KvItemDTO,
  KvKey,
  KvKeyPairWriteInput,
  KvKeys,
  KvKeysDTO,
  KvKeysListInput,
  KvPair,
  KvPairCreateInput,
  KvPairGetInput,
  KvPairsDeleteInput,
  KvPairsDeleteResult,
} from '@/features/kv/kv-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { convertPlainToKvErrorClass } from '@/features/kv/lib/kv-utils.ts';

export function useKvKeys(namespaceId: string) {
  const { account } = useAuth();
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialLoading, setIsInitialLoading] = useState(true);
  const [isLoadingNextKeys, setIsLoadingNextKeys] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [kvKeys, setKvKeys] = useState<KvKeys | null>(null);
  const [deletionResult, setDeletionResult] = useState<KvPairsDeleteResult | null>(null);
  const [prefix, setPrefix] = useState<string | undefined>(undefined);
  const [hasNextKeys, setHasNextKeys] = useState<boolean>(false);
  const [error, setError] = useState<KvError | null>(null);

  const loadKeys = async (cursor?: string, limit?: number, prefix?: string) => {
    setIsLoading(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const nextKeys = await invokeListKvKeys(
        { account_id: account?.id ?? '', namespace_id: namespaceId, cursor, limit, prefix },
        credentials,
      );
      if (cursor) {
        setKvKeys((previousKeys) => {
          return {
            keys: [...(previousKeys?.keys ?? []), ...nextKeys.keys],
            cursor: nextKeys.cursor,
          };
        });
      } else {
        setKvKeys(nextKeys);
      }

      setHasNextKeys(!!nextKeys.cursor);
      setError(null);
    } catch (e) {
      console.error(e);
      setError(e as KvError);
    } finally {
      setIsLoading(false);
    }
  };

  const loadKeysInitial = async () => {
    try {
      setIsInitialLoading(true);
      setKvKeys(null);
      await loadKeys(undefined, undefined, prefix);
    } finally {
      setIsInitialLoading(false);
    }
  };

  const loadNextKeys = async () => {
    try {
      setIsLoadingNextKeys(true);
      await loadKeys(kvKeys?.cursor);
    } finally {
      setIsLoadingNextKeys(false);
    }
  };

  const reloadKeys = async () => {
    const limit = kvKeys?.keys.length;
    await loadKeys(undefined, limit, prefix);
  };

  const setKey = (keyToReplace: KvKey) => {
    setKvKeys((previousKeys) => {
      if (!previousKeys) {
        return null;
      }

      return {
        keys: previousKeys?.keys.map((key) => (key.name === keyToReplace.name ? keyToReplace : key)),
        cursor: previousKeys?.cursor,
      };
    });
  };

  const deleteKeys = async (keys: string[]) => {
    setIsDeleting(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };
    const input: KvPairsDeleteInput = {
      account_id: account?.id ?? '',
      namespace_id: namespaceId,
      keys,
    };

    try {
      const result = await invokeDeleteKvPairs(input, credentials);
      setDeletionResult(result);
    } catch (e) {
      console.error(e);
      setError(e as KvError);
    } finally {
      setIsDeleting(false);
    }
  };

  useEffect(() => {
    loadKeysInitial().then();
  }, [namespaceId, prefix]);

  return {
    kvKeys,
    prefix,
    deletionResult,
    isLoading,
    isInitialLoading,
    isLoadingNextKeys,
    isDeleting,
    hasNextKeys,
    loadNextKeys,
    reloadKeys,
    setKey,
    setPrefix,
    deleteKeys,
    error,
  };
}

export function useKvItem() {
  const { account } = useAuth();
  const [kvItem, setKvItem] = useState<KvPair | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isWriting, setIsWriting] = useState(false);
  const [error, setError] = useState<KvError | null>(null);

  const getKvPair = async (namespaceId: string, key: string) => {
    setIsLoading(true);
    setKvItem(null);

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
      setKvItem(item);
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
      const createdKvItem = await invokeCreateKvPair({ ...input, account_id: account?.id ?? '' }, credentials);
      setKvItem(createdKvItem);
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
      const updatedKvItem = await invokeWriteKvPair({ ...input, account_id: account?.id ?? '' }, credentials);
      setKvItem(updatedKvItem);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsWriting(false);
    }
  };

  return {
    kvItem,
    getKvPair,
    createKvPair,
    writeKvPair,
    isLoading,
    isCreating,
    isWriting,
    error,
  };
}

export async function invokeListKvKeys(input: KvKeysListInput, credentials: UserAuthTokenCredentials): Promise<KvKeys> {
  try {
    const kvKeys = await invoke<KvKeysDTO>('list_kv_keys', {
      input,
      credentials,
    });

    return {
      ...kvKeys,
      keys: kvKeys.keys.map((key) => ({
        ...key,
        expiration: key.expiration ? new Date(key.expiration * 1000) : undefined,
      })),
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeGetKvPair(input: KvPairGetInput, credentials: UserAuthTokenCredentials): Promise<KvPair> {
  try {
    const kvItem = await invoke<KvItemDTO>('get_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration * 1000) : undefined,
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
    const kvItem = await invoke<KvItemDTO>('create_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration * 1000) : undefined,
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
    const kvItem = await invoke<KvItemDTO>('write_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeDeleteKvPairs(
  input: KvPairsDeleteInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPairsDeleteResult> {
  try {
    return invoke<KvPairsDeleteResult>('delete_kv_pairs', {
      input,
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}
