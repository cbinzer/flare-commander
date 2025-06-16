import { Cell, Header } from '@tanstack/react-table';
import { KvTableKey } from '@/features/kv/kv-models.ts';

export function calcElementWidth(element: Cell<KvTableKey, unknown> | Header<KvTableKey, unknown>): string | undefined {
  const meta = element.column.columnDef.meta as { width?: string };
  if (meta) {
    return meta.width;
  }

  return undefined;
}
