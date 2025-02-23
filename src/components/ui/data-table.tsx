import {
  Cell,
  ColumnDef,
  flexRender,
  getCoreRowModel,
  Header,
  PaginationState,
  useReactTable,
} from '@tanstack/react-table';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { Skeleton } from '@/components/ui/skeleton.tsx';

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  onPaginationChange?: (pagination: PaginationState) => void;
  hasNextPage?: boolean;
  isLoading?: boolean;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  onPaginationChange = () => {},
  hasNextPage = false,
  isLoading = false,
}: DataTableProps<TData, TValue>) {
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });
  const [rowSelection, setRowSelection] = useState({});

  useEffect(() => {
    onPaginationChange(pagination);
  }, [pagination]);

  const table = useReactTable({
    data,
    columns,
    onPaginationChange: setPagination,
    onRowSelectionChange: setRowSelection,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    state: {
      pagination,
      rowSelection,
    },
  });

  const getCustomWidth = (
    item: Header<TData, unknown> | Cell<TData, unknown>,
  ) => {
    const meta = item.column.columnDef.meta as
      | { width: string | undefined }
      | undefined;
    if (meta?.width) {
      return meta.width;
    }

    return undefined;
  };

  return (
    <div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead
                      key={header.id}
                      style={{
                        width: getCustomWidth(header),
                      }}
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>

          {isLoading ? (
            <TableBody>
              {Array(pagination.pageSize)
                .fill(null)
                .map((_, i) => (
                  <TableRow key={i}>
                    <TableCell>
                      <Skeleton className="h-5" />
                    </TableCell>
                    <TableCell>
                      <Skeleton className="h-5" />
                    </TableCell>
                    <TableCell>
                      <Skeleton className="h-5" />
                    </TableCell>
                  </TableRow>
                ))}
            </TableBody>
          ) : (
            <TableBody>
              {table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow
                    key={row.id}
                    data-state={row.getIsSelected() && 'selected'}
                  >
                    {row.getVisibleCells().map((cell) => (
                      <TableCell
                        key={cell.id}
                        style={{ width: getCustomWidth(cell) }}
                      >
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className="h-24 text-center"
                  >
                    No results.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          )}
        </Table>
      </div>

      <div className="flex items-center justify-end space-x-2 py-4">
        <Button
          variant="outline"
          size="sm"
          onClick={async () => {
            table.previousPage();
          }}
          disabled={!table.getCanPreviousPage() || isLoading}
        >
          Previous
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={async () => {
            table.nextPage();
          }}
          disabled={!hasNextPage || isLoading}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
