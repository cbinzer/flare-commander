import { useState } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import { invoke } from '@tauri-apps/api/core';
import { KvError, KvNamespace } from '@/kv/kv-models.ts';

export function useNamespaces() {
  const { account } = useAuth();
  const [namespaces, setNamespaces] = useState<KvNamespace[] | null>(null);
  const [loading, setLoading] = useState(false);

  const getNamespaces = async () => {
    setLoading(true);

    try {
      const namespaces = await invoke<KvNamespace[]>('get_namespaces', {
        credentials: {
          type: 'UserAuthToken',
          account_id: account?.id,
          token: account?.token.value,
        },
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
