import { useEffect, useState } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import { invoke } from '@tauri-apps/api/core';
import { KvError, KvItem, KvItems, KvKeys, KvNamespace } from '@/kv/kv-models.ts';
import { CredentialsType, UserAuthTokenCredentials } from '@/authentication/auth-models.ts';

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

export function useKvItems(namespaceId: string) {
  const { account } = useAuth();
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialLoading, setIsInitialLoading] = useState(true);
  const [isLoadingNextItems, setIsLoadingNextItems] = useState(false);
  const [kvItems, setKvItems] = useState<KvItems | null>(null);
  const [hasNextItems, setHasNextItems] = useState<boolean>(false);
  const [error, setError] = useState<KvError | null>(null);

  const loadItems = async (cursor?: string) => {
    setIsLoading(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const items = await getKvItems({ namespaceId, cursor }, credentials);
      setKvItems((previousItems) => {
        return {
          items: [...(previousItems?.items ?? []), ...items.items],
          cursor: items.cursor,
        };
      });

      setHasNextItems(!!items.cursor);
      setError(null);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoading(false);
    }
  };

  const loadItemsInitial = async () => {
    try {
      setIsInitialLoading(true);
      setKvItems(null);
      await loadItems();
    } finally {
      setIsInitialLoading(false);
    }
  };

  const loadNextItems = async () => {
    try {
      setIsLoadingNextItems(true);
      await loadItems(kvItems?.cursor);
    } finally {
      setIsLoadingNextItems(false);
    }
  };

  useEffect(() => {
    loadItemsInitial().then();
  }, [namespaceId]);

  return {
    kvItems,
    isLoading,
    isInitialLoading,
    isLoadingNextItems,
    hasNextItems,
    loadNextItems,
    error,
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

  const loadKeys = async (cursor?: string) => {
    setIsLoading(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const nextKeys = await getKvKeys({ namespaceId, cursor }, credentials);
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
    error,
  };
}

export function useKvItem(namespaceId: string, key: string) {
  const { account } = useAuth();
  const [kvItem, setKvItem] = useState<KvItem | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<KvError | null>(null);

  const loadItem = async () => {
    setIsLoading(true);
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

  useEffect(() => {
    loadItem().then();
  }, [namespaceId, key]);

  return {
    kvItem,
    isLoading,
    error,
  };
}
async function getKvItems(
  input: {
    namespaceId: string;
    cursor?: string;
  },
  credentials: UserAuthTokenCredentials,
): Promise<KvItems> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      cursor: input.cursor,
    };

    const kvItems = await invoke<KvItems>('get_kv_items', {
      input: invokeInput,
      credentials,
    });

    return {
      ...kvItems,
      items: kvItems.items.map((item) => ({
        ...item,
        expiration: item.expiration ? new Date(item.expiration) : undefined,
      })),
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function getKvKeys(
  input: {
    namespaceId: string;
    cursor?: string;
  },
  credentials: UserAuthTokenCredentials,
): Promise<KvKeys> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      cursor: input.cursor,
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
