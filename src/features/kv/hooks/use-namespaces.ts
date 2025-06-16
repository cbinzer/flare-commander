import { useAuth } from '@/features/authentication/hooks/use-auth.ts';
import { useState } from 'react';
import {
  KvError,
  KvNamespace,
  KvNamespaceCreateInput,
  KvNamespaceDeleteInput,
  KvNamespaceGetInput,
  KvNamespaces,
  KvNamespacesListInput,
  KvNamespacesOrderBy,
  KvNamespaceUpdateInput,
} from '@/features/kv/kv-models.ts';
import { CredentialsType, UserAuthTokenCredentials } from '@/features/authentication/auth-models.ts';

import { invoke } from '@tauri-apps/api/core';
import { convertPlainToKvErrorClass } from '@/features/kv/lib/kv-utils.ts';

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
