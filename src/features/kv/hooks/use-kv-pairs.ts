import { KvError, KvPair, KvPairDTO, KvPairsGetInput } from '@/features/kv/kv-models.ts';
import { Credentials } from '@/features/authentication/auth-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { convertPlainToKvErrorClass } from '@/features/kv/lib/kv-utils.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import { useState } from 'react';

export function useKvPairs() {
  const { account } = useAuth();
  const [isExporting, setIsExporting] = useState(false);
  const [kvPairsExport, setKvPairsExport] = useState<Uint8Array | null>(null);

  const exportKvPairs = async (namespaceId: string, keys: string[]): Promise<Uint8Array> => {
    setIsExporting(true);
    setKvPairsExport(null);

    try {
      const kvPairs = await invokeGetKvPairs(
        {
          account_id: account?.id as string,
          namespace_id: namespaceId,
          keys,
        },
        account?.credentials as Credentials,
      );
      const kvPairsJSON = JSON.stringify(kvPairs, (_, value) => {
        if (value instanceof Uint8Array) {
          return Array.from(value);
        }

        return value;
      });
      const encoder = new TextEncoder();
      const kvPairsExport = encoder.encode(kvPairsJSON);

      setKvPairsExport(kvPairsExport);

      return kvPairsExport;
    } finally {
      setIsExporting(false);
    }
  };

  return {
    exportKvPairs,
    isExporting,
    kvPairsExport,
  };
}

export async function invokeGetKvPairs(input: KvPairsGetInput, credentials: Credentials): Promise<KvPair[]> {
  try {
    const kvPairs = await invoke<KvPairDTO[]>('get_kv_pairs', {
      input,
      credentials,
    });

    return kvPairs.map((kvPair) => ({
      ...kvPair,
      value: new Uint8Array(kvPair.value ?? []),
      expiration: kvPair.expiration ? new Date(kvPair.expiration * 1000) : undefined,
    }));
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}
