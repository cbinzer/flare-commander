import { CredentialsType, UserAuthTokenCredentials } from '@/authentication/auth-models.ts';
import { useAuth } from '@/authentication/use-auth.ts';
import {
  CreateKvItemInput,
  KvError,
  KvItem,
  KvItemDTO,
  KvItemsDeletionInput,
  KvItemsDeletionResult,
  KvKey,
  KvKeys,
  KvKeysDTO,
  KvNamespace,
  WriteKvItemInput,
} from '@/kv/kv-models.ts';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export function useNamespaces() {
  const { account } = useAuth();
  const [isListing, setIsListing] = useState(false);
  const [isRelisting, setIsRelisting] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [namespaces, setNamespaces] = useState<KvNamespace[] | null>(null);
  const [namespace, setNamespace] = useState<KvNamespace | null>(null);
  const [error, setError] = useState<KvError | null>(null);

  const listNamespaces = async () => {
    setIsListing(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const namespaces = await invokeListNamespaces(credentials);
      setNamespaces(namespaces);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsListing(false);
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
      const namespaces = await invokeListNamespaces(credentials);
      setNamespaces(namespaces);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsRelisting(false);
    }
  };

  const createNamespace = async (title: string) => {
    setIsCreating(true);

    try {
      const credentials: UserAuthTokenCredentials = {
        type: CredentialsType.UserAuthToken,
        account_id: account?.id ?? '',
        token: (account?.credentials as UserAuthTokenCredentials).token,
      };
      const createdNamespace = await invokeCreateNamespace(title, credentials);
      setNamespace(createdNamespace);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsCreating(false);
    }
  };

  return {
    isListing,
    isRelisting,
    isCreating,
    namespace,
    namespaces,
    error,
    listNamespaces,
    relistNamespaces,
    createNamespace,
    setNamespaces,
  };
}

export function invokeListNamespaces(credentials: UserAuthTokenCredentials): Promise<KvNamespace[]> {
  try {
    return invoke<KvNamespace[]>('get_namespaces', {
      credentials,
    });
  } catch (e) {
    throw convertPlainToKvErrorClass(e as KvError);
  }
}

export function invokeCreateNamespace(title: string, credentials: UserAuthTokenCredentials): Promise<KvNamespace> {
  try {
    return invoke<KvNamespace>('create_namespace', {
      title,
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
  const [deletionResult, setDeletionResult] = useState<KvItemsDeletionResult | null>(null);
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
      const nextKeys = await getKvKeys({ namespaceId, cursor, limit, prefix }, credentials);
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
    const input: KvItemsDeletionInput = {
      namespace_id: namespaceId,
      keys,
    };

    try {
      const result = await deleteKvItems(input, credentials);
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
  const [kvItem, setKvItem] = useState<KvItem | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isWriting, setIsWriting] = useState(false);
  const [error, setError] = useState<KvError | null>(null);

  const loadKvItem = async (namespaceId: string, key: string) => {
    setIsLoading(true);
    setKvItem(null);

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

  const addKvItem = async (input: CreateKvItemInput) => {
    setIsCreating(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };
    try {
      const createdKvItem = await createKvItem(input, credentials);
      setKvItem(createdKvItem);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsCreating(false);
    }
  };

  const upsertKvItem = async (input: WriteKvItemInput) => {
    setIsWriting(true);

    const credentials: UserAuthTokenCredentials = {
      type: CredentialsType.UserAuthToken,
      account_id: account?.id ?? '',
      token: (account?.credentials as UserAuthTokenCredentials).token,
    };

    try {
      const updatedKvItem = await writeKvItem(input, credentials);
      setKvItem(updatedKvItem);
    } catch (e) {
      setError(e as KvError);
    } finally {
      setIsWriting(false);
    }
  };

  return {
    kvItem,
    loadKvItem,
    createKvItem: addKvItem,
    writeKvItem: upsertKvItem,
    isLoading,
    isCreating,
    isWriting,
    error,
  };
}

export async function getKvKeys(
  input: {
    namespaceId: string;
    cursor?: string;
    limit?: number;
    prefix?: string;
  },
  credentials: UserAuthTokenCredentials,
): Promise<KvKeys> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      cursor: input.cursor,
      limit: input.limit,
      prefix: input.prefix,
    };

    const kvKeys = await invoke<KvKeysDTO>('get_kv_keys', {
      input: invokeInput,
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

    const kvItem = await invoke<KvItemDTO>('get_kv_item', {
      input: invokeInput,
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

export async function createKvItem(input: CreateKvItemInput, credentials: UserAuthTokenCredentials): Promise<KvItem> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      key: input.key,
      value: input.value,
      expiration: input.expiration,
      metadata: input.metadata,
    };

    const kvItem = await invoke<KvItemDTO>('create_kv_item', {
      input: invokeInput,
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

export async function writeKvItem(input: WriteKvItemInput, credentials: UserAuthTokenCredentials): Promise<KvItem> {
  try {
    const invokeInput = {
      namespace_id: input.namespaceId,
      key: input.key,
      value: input.value,
      expiration: input.expiration,
      metadata: input.metadata,
    };

    const kvItem = await invoke<KvItemDTO>('write_kv_item', {
      input: invokeInput,
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

export async function deleteKvItems(
  input: KvItemsDeletionInput,
  credentials: UserAuthTokenCredentials,
): Promise<KvItemsDeletionResult> {
  try {
    return invoke<KvItemsDeletionResult>('delete_kv_items', {
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
