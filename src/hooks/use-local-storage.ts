import { Dispatch, SetStateAction, useEffect, useState } from 'react';

export function useLocalStorage<T>(
  storageKey: string,
  fallbackState: T | null = null,
): [T | null, Dispatch<SetStateAction<T | null>>] {
  const [value, setValue] = useState<T | null>(getItemFromLocalStorage<T>(storageKey) ?? fallbackState);

  useEffect(() => localStorage.setItem(storageKey, JSON.stringify(value)), [value]);
  useEffect(() => setValue(getItemFromLocalStorage<T>(storageKey) ?? fallbackState), [storageKey]);

  return [value, setValue];
}

function getItemFromLocalStorage<T>(key: string): T | null {
  const item = localStorage.getItem(key);
  if (item) {
    return JSON.parse(item) as T;
  }

  return null;
}
