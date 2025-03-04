import { Cell, Header } from '@tanstack/react-table';
import { KvItem } from '@/kv/kv-models.ts';

export function calcElementWidth(
  element: Cell<KvItem, unknown> | Header<KvItem, unknown>,
): string | undefined {
  const meta = element.column.columnDef.meta as { width?: string };
  if (meta) {
    return meta.width;
  }

  return undefined;
}
