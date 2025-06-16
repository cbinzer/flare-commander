import { flexRender, Row } from '@tanstack/react-table';
import { TableBody, TableCell, TableRow } from '@/components/ui/table.tsx';
import { KvTableKey } from '@/features/kv/kv-models.ts';
import { calcElementWidth } from '@/features/kv/lib/kv-table-utils.ts';

interface KvTableBodyProps {
  rows: Row<KvTableKey>[];
  amountItems?: number;
  onRowClick?: (row: Row<KvTableKey>) => void;
}

export function KvTableBody(props: KvTableBodyProps) {
  return (
    <TableBody>
      {props.rows?.map((row) => (
        <TableRow
          key={row.id}
          data-state={row.getIsSelected() && 'selected'}
          className="h-[40px]"
          onClick={() => props.onRowClick?.(row)}
        >
          {row.getVisibleCells().map((cell) => (
            <TableCell key={cell.id} style={{ width: calcElementWidth(cell) }}>
              {flexRender(cell.column.columnDef.cell, cell.getContext())}
            </TableCell>
          ))}
        </TableRow>
      ))}
    </TableBody>
  );
}
