import { Bucket, BucketsDTO, BucketsListInput } from './r2-models.ts';
import { useCallback, useState } from 'react';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import { Credentials } from '@/features/authentication/auth-models.ts';
import { invoke } from '@tauri-apps/api/core';

export function useBuckets(): BucketsHook {
  const auth = useAuth();
  const [loading, setLoading] = useState<boolean>(false);
  const [buckets, setBuckets] = useState<Bucket[]>([]);

  const loadBuckets = useCallback(async () => {
    setLoading(true);

    const credentials = auth.account?.credentials;
    if (!credentials) {
      setLoading(false);
      auth.resetCredentials();
      return;
    }

    try {
      const bucketsResponse = await invokeListBuckets(credentials, { account_id: auth.account?.id ?? '' });
      setBuckets(bucketsResponse.items.map((bucket) => ({ ...bucket, creation_date: new Date(bucket.creation_date) })));
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
    }
  }, [auth]);

  return {
    loading,
    buckets,
    loadBuckets,
  };
}

export interface BucketsHook {
  loading: boolean;
  buckets: Bucket[];
  loadBuckets: () => Promise<void>;
}

async function invokeListBuckets(credentials: Credentials, input: BucketsListInput): Promise<BucketsDTO> {
  return invoke<BucketsDTO>('list_buckets', {
    credentials,
    input,
  });
}
