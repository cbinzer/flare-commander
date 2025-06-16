import { CredentialsType, UserAuthTokenCredentials } from '@/features/authentication/auth-models.ts';
import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import {
  KvError,
  KvItemDTO,
  KvKey,
  KvKeyPairWriteInput,
  KvKeys,
  KvKeysDTO,
  KvKeysListInput,
  KvNamespace,
  KvNamespaceCreateInput,
  KvNamespaceDeleteInput,
  KvNamespaceGetInput,
  KvNamespaces,
  KvNamespacesListInput,
  KvNamespacesOrderBy,
  KvNamespaceUpdateInput,
  KvPair,
  KvPairCreateInput,
  KvPairGetInput,
  KvPairsDeleteInput,
  KvPairsDeleteResult,
} from '@/features/kv/kv-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export function useNamespaces() {
  const { account } = useAuth();
  const [isListing, setIsListing] = useState(false);
  const [isRelisting, setIsRelisting] = useState(false);
  const [isLoadingOne, setIsLoadingOne] = useState(false);
  const [isLoadingNext, setIsLoadingNext] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [namespaces, setNamespaces] = useState<KvNamespace[] | null>(null);
  const [namespace, setNamespace] = useState<KvNamespace | null>(null);
  const [page, setPage] = useState<number>(1);
  const [totalCount, setTotalCount] = useState<number>(0);
  const [error, setError] = useState<KvError | null>(null);

  const listNamespaces = async () => {
    setIsListing(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const namespaces = await invokeListNamespaces(credentials, {
        account_id: account?.id ?? '',
        order_by: KvNamespacesOrderBy.TITLE,
      });

      setPage(namespaces.page_info.page);
      setTotalCount(namespaces.page_info.total_count);
      setNamespaces(namespaces.items);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsListing(false);
    }
  };

  const listNextNamespaces = async () => {
    setIsLoadingNext(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const nextNamespaces = await invokeListNamespaces(credentials, {
        account_id: account?.id ?? '',
        order_by: KvNamespacesOrderBy.TITLE,
        page: page + 1,
      });
      const allNamespaceIds = new Set(
        [...(namespaces ?? []), ...nextNamespaces.items].map((namespace) => namespace.id),
      );
      const allNamespaces: KvNamespace[] = [];
      allNamespaceIds.forEach((namespaceId) => {
        const newNamespace = nextNamespaces.items.find((namespace) => namespace.id === namespaceId);
        if (newNamespace) {
          allNamespaces.push(newNamespace);
          return;
        }

        const existingNamespace = namespaces?.find((namespace) => namespace.id === namespaceId);
        if (existingNamespace) {
          allNamespaces.push(existingNamespace);
        }
      });

      setPage(nextNamespaces.page_info.page);
      setTotalCount(nextNamespaces.page_info.total_count);
      setNamespaces(allNamespaces);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoadingNext(false);
    }
  };

  const relistNamespaces = async () => {
    setIsRelisting(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const reloadedNamespaces = await invokeListNamespaces(credentials, {
        account_id: account?.id ?? '',
        order_by: KvNamespacesOrderBy.TITLE,
        per_page: namespaces?.length ?? 20,
      });

      setPage(reloadedNamespaces.page_info.page);
      setTotalCount(reloadedNamespaces.page_info.total_count);
      setNamespaces(reloadedNamespaces.items);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsRelisting(false);
    }
  };

  const getNamespace = async (namespaceId: string) => {
    setIsLoadingOne(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const input: KvNamespaceGetInput = {
        account_id: account?.id ?? '',
        namespace_id: namespaceId,
      };
      const namespace = await invokeGetNamespace(input, credentials);
      setNamespace(namespace);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoadingOne(false);
    }
  };

  const createNamespace = async (input: Omit<KvNamespaceCreateInput, 'account_id'>) => {
    setIsCreating(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const createdNamespace = await invokeCreateNamespace({ ...input, account_id: account?.id ?? '' }, credentials);
      setNamespace(createdNamespace);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsCreating(false);
    }
  };

  const updateNamespace = async (input: Omit<KvNamespaceUpdateInput, 'account_id'>) => {
    setIsUpdating(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      await invokeUpdateNamespace({ ...input, account_id: account?.id ?? '' }, credentials);
    } catch (e) {
      setError(e as KvError);
      throw e;
    } finally {
      setIsUpdating(false);
    }
  };

  const deleteNamespace = async (namespaceId: string) => {
    setIsDeleting(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      await invokeDeleteNamespace({ account_id: account?.id ?? '', namespace_id: namespaceId }, credentials);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsDeleting(false);
    }
  };

  return {
    isListing,
    isLoadingNext,
    isRelisting,
    isLoadingOne,
    isCreating,
    isUpdating,
    isDeleting,
    namespace,
    namespaces,
    totalCount,
    error,
    listNamespaces,
    listNextNamespaces,
    relistNamespaces,
    getNamespace,
    createNamespace,
    updateNamespace,
    deleteNamespace,
    setNamespaces,
  };
}

export async function invokeListNamespaces(
  credentials: UserAuthTokenCredentials,
  input: KvNamespacesListInput,
): Promise<KvNamespaces> {
  try {
    return await invoke<KvNamespaces>('list_namespaces', {
      credentials,
      input,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

export async function invokeGetNamespace(
  input: KvNamespaceGetInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvNamespace> {
  try {
    return await invoke<KvNamespace>('get_namespace', {
      credentials,
      input,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

export async function invokeCreateNamespace(
  input: KvNamespaceCreateInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvNamespace> {
  try {
    return await invoke<KvNamespace>('create_namespace', {
      input,
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

export async function invokeUpdateNamespace(
  input: KvNamespaceUpdateInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvNamespace> {
  try {
    return await invoke<KvNamespace>('update_namespace', {
      input,
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

async function invokeDeleteNamespace(input: KvNamespaceDeleteInput, credentials: UserAuthTokenCredentials) {
  try {
    return await invoke<KvNamespace>('delete_namespace', {
      input,
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

export function useKvKeys(namespaceId: string) {
  const { account } = useAuth();
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialLoading, setIsInitialLoading] = useState(true);
  const [isLoadingNextKeys, setIsLoadingNextKeys] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [kvKeys, setKvKeys] = useState<KvKeys | null>(null);
  const [deletionResult, setDeletionResult] = useState<KvPairsDeleteResult | null>(null);
  const [prefix, setPrefix] = useState<string | undefined>(undefined);
  const [hasNextKeys, setHasNextKeys] = useState<boolean>(false);
  const [error, setError] = useState<KvError | null>(null);

  const loadKeys = async (cursor?: string, limit?: number, prefix?: string) => {
    setIsLoading(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const nextKeys = await invokeListKvKeys(
        { account_id: account?.id ?? '', namespace_id: namespaceId, cursor, limit, prefix },
        credentials,
      );
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
      await loadKeys(undefined, undefined, prefix);
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
    await loadKeys(undefined, limit, prefix);
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

  const deleteKeys = async (keys: string[]) => {
    setIsDeleting(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };
    const input: KvPairsDeleteInput = {
      account_id: account?.id ?? '',
      namespace_id: namespaceId,
      keys,
    };

    try {
      const result = await invokeDeleteKvPairs(input, credentials);
      setDeletionResult(result);
    } catch (e) {
      console.error(e);
      setError(e as KvError);
    } finally {
      setIsDeleting(false);
    }
  };

  useEffect(() => {
    loadKeysInitial().then();
  }, [namespaceId, prefix]);

  return {
    kvKeys,
    prefix,
    deletionResult,
    isLoading,
    isInitialLoading,
    isLoadingNextKeys,
    isDeleting,
    hasNextKeys,
    loadNextKeys,
    reloadKeys,
    setKey,
    setPrefix,
    deleteKeys,
    error,
  };
}

export function useKvItem() {
  const { account } = useAuth();
  const [kvItem, setKvItem] = useState<KvPair | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isWriting, setIsWriting] = useState(false);
  const [error, setError] = useState<KvError | null>(null);

  const getKvPair = async (namespaceId: string, key: string) => {
    setIsLoading(true);
    setKvItem(null);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const item = await invokeGetKvPair(
        { account_id: account?.id ?? '', namespace_id: namespaceId, key },
        credentials,
      );
      setKvItem(item);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsLoading(false);
    }
  };

  const createKvPair = async (input: Omit<KvPairCreateInput, 'account_id'>) => {
    setIsCreating(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };
    try {
      const createdKvItem = await invokeCreateKvPair({ ...input, account_id: account?.id ?? '' }, credentials);
      setKvItem(createdKvItem);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsCreating(false);
    }
  };

  const writeKvPair = async (input: Omit<KvKeyPairWriteInput, 'account_id'>) => {
    setIsWriting(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const updatedKvItem = await invokeWriteKvPair({ ...input, account_id: account?.id ?? '' }, credentials);
      setKvItem(updatedKvItem);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsWriting(false);
    }
  };

  return {
    kvItem,
    getKvPair,
    createKvPair,
    writeKvPair,
    isLoading,
    isCreating,
    isWriting,
    error,
  };
}

export async function invokeListKvKeys(input: KvKeysListInput, credentials: UserAuthTokenCredentials): Promise<KvKeys> {
  try {
    const kvKeys = await invoke<KvKeysDTO>('list_kv_keys', {
      input,
      credentials,
    });

    return {
      ...kvKeys,
      keys: kvKeys.keys.map((key) => ({
        ...key,
        expiration: key.expiration ? new Date(key.expiration * 1000) : undefined,
      })),
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeGetKvPair(input: KvPairGetInput, credentials: UserAuthTokenCredentials): Promise<KvPair> {
  try {
    const kvItem = await invoke<KvItemDTO>('get_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeCreateKvPair(
  input: KvPairCreateInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPair> {
  try {
    const kvItem = await invoke<KvItemDTO>('create_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeWriteKvPair(
  input: KvKeyPairWriteInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPair> {
  try {
    const kvItem = await invoke<KvItemDTO>('write_kv_pair', {
      input,
      credentials,
    });

    return {
      ...kvItem,
      expiration: kvItem.expiration ? new Date(kvItem.expiration * 1000) : undefined,
    };
  } catch (e) {
    const kvError = e as KvError;
    console.error(kvError);
    throw new KvError(kvError.message, kvError.kind);
  }
}

export async function invokeDeleteKvPairs(
  input: KvPairsDeleteInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvPairsDeleteResult> {
  try {
    return invoke<KvPairsDeleteResult>('delete_kv_pairs', {
      input,
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

function convertPlainToKvErrorClass(kvError: KvError): KvError {
  console.error(kvError);
  return new KvError(kvError.message, kvError.kind);
}
