import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import {
  Table,
  TableBody,
  TableCell,
  TableRow,
} from '@/components/ui/table.tsx';
import { FunctionComponent, useState } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';
import { KvTableHeader } from '@/kv/table/kv-table-header.tsx';
import { KvTableBody } from '@/kv/table/kv-table-body.tsx';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import { KvItem, KvNamespace } from '@/kv/kv-models.ts';
import { useKvItems } from '@/kv/kv-hooks.ts';

interface KvTableProps {
  namespace: KvNamespace;
}

const columns: ColumnDef<KvItem>[] = [
  {
    id: 'select',
    header: ({ table }) => (
      <Checkbox
        checked={
          table.getIsAllPageRowsSelected() ||
          (table.getIsSomePageRowsSelected() && 'indeterminate')
        }
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
      width: '60px',
    },
  },
  {
    id: 'key',
    accessorKey: 'key',
    header: 'Key',
  },
  {
    id: 'value',
    accessorKey: 'value',
    header: 'Value',
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

  const {
    kvItems,
    isInitialLoading,
    isLoadingNextItems,
    hasNextItems,
    loadNextItems,
  } = useKvItems(namespace.id);

  const table = useReactTable({
    data: kvItems?.items ?? [],
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
        <Table style={{ tableLayout: 'fixed' }}>
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

      {hasNextItems && !isInitialLoading ? (
        <div className="flex items-center justify-center space-x-2 py-4">
          <Button
            variant="outline"
            size="sm"
            onClick={loadNextItems}
            disabled={isLoadingNextItems}
          >
            {isLoadingNextItems ? <LoadingSpinner /> : 'Load more'}
          </Button>
        </div>
      ) : null}
    </>
  );
}

const LoadingTableBody: FunctionComponent<{ pageSize?: number }> = ({
  pageSize = 10,
}) => {
  return (
    <TableBody>
      {Array(pageSize)
        .fill(null)
        .map((_, i) => (
          <TableRow key={i} className={'h-[40px]'}>
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

const EmptyTableBody: FunctionComponent<{ columnsLength: number }> = ({
  columnsLength,
}) => {
  return (
    <TableBody>
      <TableRow>
        <TableCell colSpan={columnsLength} className="h-24 text-center">
          No items available.
        </TableCell>
      </TableRow>
    </TableBody>
  );
};
