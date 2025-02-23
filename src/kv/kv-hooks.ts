import { useEffect, useState } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import { invoke } from '@tauri-apps/api/core';
import { KvError, KvItems, KvNamespace } from '@/kv/kv-models.ts';
import {
  CredentialsType,
  UserAuthTokenCredentials,
} from '@/authentication/auth-models.ts';

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
  const [kvItems, setKvItems] = useState<KvItems | null>(null);
  const [hasNextItems, setHasNextItems] = useState<boolean>(false);
  const [previousCursors, setPreviousCursors] = useState<string[]>([]);
  const [error, setError] = useState<KvError | null>(null);

  const loadItems = async (cursor?: string) => {
    setIsLoading(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    getKvItems({ namespaceId, cursor }, credentials)
      .then((items) => {
        setError(null);
        setKvItems(items);
        setHasNextItems(!!items.cursor);
        setIsLoading(false);
      })
      .catch((error) => {
        setError(error);
        setHasNextItems(false);
        setIsLoading(false);
      });
  };

  const loadPreviousItems = async () => {
    let previousCursor: string | undefined = undefined;
    if (previousCursors.length > 1) {
      // We need the penultimate cursor because the last one is the cursor from
      // the current page
      previousCursor = previousCursors[previousCursors.length - 2];
    }

    await loadItems(previousCursor);

    setPreviousCursors(previousCursors.slice(0, -1));
  };

  const loadNextItems = async () => {
    const cursorForNexItems = kvItems?.cursor;
    await loadItems(cursorForNexItems);

    if (cursorForNexItems) {
      setPreviousCursors([...previousCursors, cursorForNexItems]);
    }
  };

  useEffect(() => {
    loadItems().then();
  }, [namespaceId]);

  return {
    kvItems,
    isLoading,
    hasNextItems,
    loadPreviousItems,
    loadNextItems,
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
