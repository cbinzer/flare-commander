import { flexRender, Row } from '@tanstack/react-table';
import { TableBody, TableCell, TableRow } from '@/components/ui/table.tsx';
import { KvKey } from '@/kv/kv-models.ts';
import { calcElementWidth } from '@/kv/table/kv-table-utils.ts';

interface KvTableBodyProps {
  rows: Row<KvKey>[];
  amountItems?: number;
  onRowClick?: (row: Row<KvKey>) => void;
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
