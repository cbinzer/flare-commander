import {
  KvError,
  KvPair,
  KvPairDTO,
  KvPairsGetInput,
  KvPairsWriteInput,
  KvPairsWriteInputPair,
  KvPairsWriteResult,
} from '@/features/kv/kv-models.ts';
import { Credentials } from '@/features/authentication/auth-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { convertPlainToKvErrorClass } from '@/features/kv/lib/kv-utils.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';

export function useKvPairs() {
  const { account } = useAuth();

  const createKvPairsJSONExport = async (namespaceId: string, keys: string[]): Promise<Uint8Array> => {
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

    return kvPairsExport;
  };

  const writeKvPairs = async (namespaceId: string, pairs: KvPairsWriteInputPair[]): Promise<KvPairsWriteResult> => {
    return invokeWriteKvPairs(
      {
        account_id: account?.id as string,
        namespace_id: namespaceId,
        pairs,
      },
      account?.credentials as Credentials,
    );
  };

  return {
    createKvPairsJSONExport,
    writeKvPairs,
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

export async function invokeWriteKvPairs(
  input: KvPairsWriteInput,
  credentials: Credentials,
): Promise<KvPairsWriteResult> {
  try {
    return await invoke<KvPairsWriteResult>('write_kv_pairs', {
      input,
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}
