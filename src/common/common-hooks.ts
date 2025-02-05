import { Dispatch, SetStateAction, useEffect, useState } from 'react';
import { useToast } from '@/hooks/use-toast.ts';
import { useAuth } from '@/authentication/use-auth.ts';

export function useLocalStorage<T>(
  storageKey: string,
  fallbackState: T | null = null,
): [T | null, Dispatch<SetStateAction<T | null>>] {
  const [value, setValue] = useState<T | null>(
    getItemFromLocalStorage<T>(storageKey) ?? fallbackState,
  );

  useEffect(() => {
    if (value) {
      localStorage.setItem(storageKey, JSON.stringify(value));
    }
  }, [value, storageKey]);

  return [value, setValue];
}

function getItemFromLocalStorage<T>(key: string): T | null {
  const item = localStorage.getItem(key);
  if (item) {
    return JSON.parse(item) as T;
  }

  return null;
}

export function useError() {
  const { toast } = useToast();
  const { logout } = useAuth();

  const handleError = (error: Error) => {
    console.error(error);
    if ('kind' in error && error.kind === 'Authentication') {
      logout();
      return;
    }

    toast({
      variant: 'destructive',
      title: 'Something went wrong.',
      description: error.message,
    });
  };

  return {
    handleError,
  };
}
