import { ColumnDef, getCoreRowModel, Row, useReactTable } from '@tanstack/react-table';
import { Table, TableBody, TableCell, TableRow } from '@/components/ui/table.tsx';
import { FunctionComponent, useState } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';
import { KvTableHeader } from '@/kv/table/kv-table-header.tsx';
import { KvTableBody } from '@/kv/table/kv-table-body.tsx';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import { KvKey, KvNamespace } from '@/kv/kv-models.ts';
import { useKvKeys } from '@/kv/kv-hooks.ts';
import { useNavigate } from 'react-router';
import { KvItemSheet } from '../kv-item-sheet';

interface KvTableProps {
  namespace: KvNamespace;
}

const columns: ColumnDef<KvKey>[] = [
  {
    id: 'select',
    header: ({ table }) => (
      <Checkbox
        checked={table.getIsAllPageRowsSelected() || (table.getIsSomePageRowsSelected() && 'indeterminate')}
        onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
        aria-label="Select all"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => row.toggleSelected(!!value)}
        aria-label="Select row"
      />
    ),
    meta: {
      width: '30px',
    },
  },
  {
    id: 'name',
    accessorKey: 'name',
    header: 'Key Name',
    cell: (cell) => <KvItemSheet name={cell.getValue() as string} />,
  },
  {
    id: 'expiration',
    accessorKey: 'expiration',
    header: 'Expiration',
    cell: (cell) => <>{cell.getValue() ?? '-'}</>,
    meta: {
      width: '150px',
    },
  },
];

export function KvTable({ namespace }: KvTableProps) {
  const [rowSelection, setRowSelection] = useState({});
  const navigate = useNavigate();

  const { kvKeys, isInitialLoading, isLoadingNextKeys, hasNextKeys, loadNextKeys } = useKvKeys(namespace.id);

  const table = useReactTable({
    data: kvKeys?.keys ?? [],
    columns,
    onRowSelectionChange: setRowSelection,
    getCoreRowModel: getCoreRowModel(),
    state: {
      rowSelection,
    },
  });

  return (
    <>
      <div className="rounded-md border">
        <Table className="table-fixed">
          <KvTableHeader headerGroups={table.getHeaderGroups()} />

          {isInitialLoading ? (
            <LoadingTableBody />
          ) : table.getRowModel().rows?.length ? (
            <KvTableBody rows={table.getRowModel().rows} />
          ) : (
            <EmptyTableBody columnsLength={columns.length} />
          )}
        </Table>
      </div>

      {hasNextKeys && !isInitialLoading ? (
        <div className="flex items-center justify-center space-x-2 py-4">
          <Button variant="outline" size="sm" onClick={loadNextKeys} disabled={isLoadingNextKeys}>
            {isLoadingNextKeys ? <LoadingSpinner /> : 'Load more'}
          </Button>
        </div>
      ) : null}
    </>
  );
}

const LoadingTableBody: FunctionComponent<{ pageSize?: number }> = ({ pageSize = 10 }) => {
  return (
    <TableBody>
      {Array(pageSize)
        .fill(null)
        .map((_, i) => (
          <TableRow key={i} className={'h-[40px]'} selectable={false}>
            {columns.map(() => (
              <TableCell>
                <Skeleton className="h-5" />
              </TableCell>
            ))}
          </TableRow>
        ))}
    </TableBody>
  );
};

const EmptyTableBody: FunctionComponent<{ columnsLength: number }> = ({ columnsLength }) => {
  return (
    <TableBody>
      <TableRow selectable={false}>
        <TableCell colSpan={columnsLength} className="h-24 text-center">
          No keys available.
        </TableCell>
      </TableRow>
    </TableBody>
  );
};
