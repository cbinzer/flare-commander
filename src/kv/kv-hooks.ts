import { useState } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import { invoke } from '@tauri-apps/api/core';
import { KvError, KvItems, KvNamespace } from '@/kv/kv-models.ts';
import {
  CredentialsType,
  UserAuthTokenCredentials,
} from '@/authentication/auth-models.ts';
import { useQuery } from '@tanstack/react-query';

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

  const getKvItems = async (): Promise<KvItems> => {
    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const kvItems = await invoke<KvItems>('get_kv_items', {
        credentials,
        input: {
          namespace_id: namespaceId,
        },
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
  };

  return useQuery({
    queryKey: ['kv-items', namespaceId],
    queryFn: getKvItems,
  });
}
