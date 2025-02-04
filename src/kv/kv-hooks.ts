import { useState } from 'react';
import { useAuth } from '@/authentication/use-auth.ts';
import { invoke } from '@tauri-apps/api/core';
import { KvNamespace } from '@/kv/kv-models.ts';

export function useNamespaces() {
  const { account } = useAuth();
  const [namespaces, setNamespaces] = useState<KvNamespace[] | null>(null);
  const [loading, setLoading] = useState(false);

  const getNamespaces = async () => {
    setLoading(true);

    const namespaces = await invoke<KvNamespace[]>('get_namespaces', {
      accountId: account?.id,
      token: account?.token.value,
    });
    setNamespaces(namespaces);

    setLoading(false);
  };

  return {
    loading,
    namespaces,
    getNamespaces,
    setNamespaces,
  };
}
