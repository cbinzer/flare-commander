import { CredentialsType, UserAuthTokenCredentials } from '@/authentication/auth-models.ts';
import { useAuth } from '@/authentication/use-auth.ts';
import { KvError, KvItem, KvKey, KvKeys, KvNamespace, WriteKvItemInput } from '@/kv/kv-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export function useNamespaces() {
  const { account } = useAuth();
  const [namespaces, setNamespaces] = useState<KvNamespace[] | null>(null);
  const [loading, setLoading] = useState(false);

  const getNamespaces = async () => {
    setLoading(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const namespaces = await invoke<KvNamespace[]>('get_namespaces', {
        credentials,
      });
      setNamespaces(namespaces);
    } catch (e) {
      const kvError = e as KvError;
      console.error(kvError);
      throw new KvError(kvError.message, kvError.kind);
    } finally {
      setLoading(false);
    }
  };

  return {
    loading,
    namespaces,
    getNamespaces,
    setNamespaces,
  };
}

export function useKvKeys(namespaceId: string) {
  const { account } = useAuth();
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialLoading, setIsInitialLoading] = useState(true);
  const [isLoadingNextKeys, setIsLoadingNextKeys] = useState(false);
  const [kvKeys, setKvKeys] = useState<KvKeys | null>(null);
  const [hasNextKeys, setHasNextKeys] = useState<boolean>(false);
  const [error, setError] = useState<KvError | null>(null);

  const loadKeys = async (cursor?: string, limit?: number) => {
    setIsLoading(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const nextKeys = await getKvKeys({ namespaceId, cursor, limit }, credentials);
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
      await loadKeys();
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
    await loadKeys(undefined, limit);
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

  useEffect(() => {
    loadKeysInitial().then();
  }, [namespaceId]);

  return {
    kvKeys,
    isLoading,
    isInitialLoading,
    isLoadingNextKeys,
    hasNextKeys,
    loadNextKeys,
    reloadKeys,
    setKey,
    error,
  };
}

export function useKvItem() {
  const { account } = useAuth();
  const [kvItem, setKvItem] = useState<KvItem | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isWriting, setIsWriting] = useState(false);
  const [error, setError] = useState<KvError | null>(null);

  const loadKvItem = async (namespaceId: string, key: string) => {
    setIsLoading(true);
    setKvItem(null);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const item = await getKvItem({ namespaceId, key }, credentials);
      setKvItem(item);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoading(false);
    }
  };

  const upsertKvItem = async (input: WriteKvItemInput) => {
    setIsWriting(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const updatedKvItem = await writeKvItem(input, credentials);
      setKvItem(updatedKvItem);
    } finally {
      setIsWriting(false);
    }
  };

  return {
    kvItem,
    loadKvItem,
    writeKvItem: upsertKvItem,
    isLoading,
    isWriting,
    error,
  };
}

export async function getKvKeys(
  input: {
    namespaceId: string;
    cursor?: string;
    limit?: number;
  },
  credentials: UserAuthTokenCredentials,
): Promise<KvKeys> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      cursor: input.cursor,
      limit: input.limit,
    };

    const kvKeys = await invoke<KvKeys>('get_kv_keys', {
      input: invokeInput,
      credentials,
    });

    return {
      ...kvKeys,
      keys: kvKeys.keys.map((key) => ({
        ...key,
        expiration: key.expiration ? new Date(key.expiration) : undefined,
      })),
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function getKvItem(
  input: {
    namespaceId: string;
    key: string;
  },
  credentials: UserAuthTokenCredentials,
): Promise<KvItem> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      key: input.key,
    };

    const kvItem = await invoke<KvItem>('get_kv_item', {
      input: invokeInput,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function writeKvItem(input: WriteKvItemInput, credentials: UserAuthTokenCredentials): Promise<KvItem> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      key: input.key,
      value: input.value,
      expiration: input.expiration,
      metadata: input.metadata,
    };

    const kvItem = await invoke<KvItem>('set_kv_item', {
      input: invokeInput,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}
