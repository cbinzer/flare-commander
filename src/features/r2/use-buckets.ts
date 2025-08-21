import { Bucket, BucketsDTO, BucketsListInput } from './r2-models.ts';
import { useCallback, useState } from 'react';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import { Credentials } from '@/features/authentication/auth-models.ts';
import { invoke } from '@tauri-apps/api/core';

const PER_PAGE = 100;

export function useBuckets(): BucketsHook {
  const auth = useAuth();
  const [loading, setLoading] = useState<boolean>(false);
  const [reloading, setReloading] = useState<boolean>(false);
  const [loadingNext, setLoadingNext] = useState<boolean>(false);
  const [hasNext, setHasNext] = useState<boolean>(false);
  const [buckets, setBuckets] = useState<Bucket[]>([]);
  const [cursor, setCursor] = useState<string | undefined>();

  const loadBuckets = useCallback(async () => {
    setLoading(true);

    const credentials = auth.account?.credentials;
    if (!credentials) {
      setLoading(false);
      auth.resetCredentials();
      return;
    }

    try {
      const bucketsResponse = await invokeListBuckets(credentials, {
        account_id: auth.account?.id ?? '',
        per_page: PER_PAGE,
      });
      setHasNext(!!bucketsResponse.page_info?.cursor);
      setCursor(bucketsResponse.page_info?.cursor);
      setBuckets(bucketsResponse.items.map((bucket) => ({ ...bucket, creation_date: new Date(bucket.creation_date) })));
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
    }
  }, [auth]);

  const reloadBuckets = useCallback(async () => {
    setLoading(true);
    setReloading(true);

    const credentials = auth.account?.credentials;
    if (!credentials) {
      setLoading(false);
      setReloading(false);
      auth.resetCredentials();
      return;
    }

    try {
      const bucketsResponse = await invokeListBuckets(credentials, {
        account_id: auth.account?.id ?? '',
        per_page: buckets.length,
      });

      setHasNext(!!bucketsResponse.page_info?.cursor);
      setCursor(bucketsResponse.page_info?.cursor);
      setBuckets(bucketsResponse.items.map((bucket) => ({ ...bucket, creation_date: new Date(bucket.creation_date) })));
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
      setReloading(false);
    }
  }, [auth, buckets]);

  const loadNextBuckets = useCallback(async () => {
    setLoading(true);
    setLoadingNext(true);

    const credentials = auth.account?.credentials;
    if (!credentials) {
      setLoading(false);
      setLoadingNext(false);
      auth.resetCredentials();
      return;
    }

    try {
      const bucketsResponse = await invokeListBuckets(credentials, {
        account_id: auth.account?.id ?? '',
        cursor,
        per_page: PER_PAGE,
      });
      setHasNext(!!bucketsResponse.page_info?.cursor);
      setCursor(bucketsResponse.page_info?.cursor);
      setBuckets((previousBuckets) => [
        ...previousBuckets,
        ...bucketsResponse.items.map((bucket) => ({ ...bucket, creation_date: new Date(bucket.creation_date) })),
      ]);
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
      setLoadingNext(false);
    }
  }, [auth, cursor]);

  return {
    loading,
    reloading,
    loadingNext,
    hasNext,
    buckets,
    loadBuckets,
    reloadBuckets,
    loadNextBuckets,
  };
}

export interface BucketsHook {
  loading: boolean;
  reloading: boolean;
  loadingNext: boolean;
  hasNext: boolean;
  buckets: Bucket[];
  loadBuckets: () => Promise<void>;
  reloadBuckets: () => Promise<void>;
  loadNextBuckets: () => Promise<void>;
}

async function invokeListBuckets(credentials: Credentials, input: BucketsListInput): Promise<BucketsDTO> {
  return invoke<BucketsDTO>('list_buckets', {
    credentials,
    input,
  });
}
