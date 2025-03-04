import { flexRender, Row } from '@tanstack/react-table';
import { TableBody, TableCell, TableRow } from '@/components/ui/table.tsx';
import { KvItem } from '@/kv/kv-models.ts';
import { calcElementWidth } from '@/kv/table/kv-table-utils.ts';

interface DataTableBodyProps {
  rows: Row<KvItem>[];
  amountItems?: number;
}

export function KvTableBody(props: DataTableBodyProps) {
  return (
    <TableBody>
      {props.rows?.map((row) => (
        <TableRow
          key={row.id}
          data-state={row.getIsSelected() && 'selected'}
          className="h-[40px]"
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
