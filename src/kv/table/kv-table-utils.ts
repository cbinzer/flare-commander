import { Cell, Header } from '@tanstack/react-table';
import { KvKey } from '@/kv/kv-models.ts';

export function calcElementWidth(
  element: Cell<KvKey, unknown> | Header<KvKey, unknown>,
): string | undefined {
  const meta = element.column.columnDef.meta as { width?: string };
  if (meta) {
    return meta.width;
  }

  return undefined;
}
