import { Button } from '@/components/ui/button.tsx';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { Table, TableBody, TableCell, TableRow } from '@/components/ui/table.tsx';
import { useKvKeys } from '@/kv/kv-hooks.ts';
import { KvNamespace, KvTableKey } from '@/kv/kv-models.ts';
import { KvTableBody } from '@/kv/table/kv-table-body.tsx';
import { KvTableHeader } from '@/kv/table/kv-table-header.tsx';
import { ColumnDef, getCoreRowModel, useReactTable } from '@tanstack/react-table';
import { format } from 'date-fns';
import { FocusEvent, FunctionComponent, KeyboardEvent, useEffect, useMemo, useState } from 'react';
import KvItemUpdateSheet from '../kv-item-update-sheet.tsx';
import {
  ArrowDown,
  EditIcon,
  MoreVerticalIcon,
  PlusIcon,
  RefreshCcwIcon,
  Search,
  Trash2Icon,
  TrashIcon,
} from 'lucide-react';
import KvItemCreateSheet from '@/kv/kv-item-create-sheet.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu.tsx';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog.tsx';
import { Input } from '@/components/ui/input.tsx';

interface KvTableProps {
  namespace: KvNamespace;
}

export function KvTable({ namespace }: KvTableProps) {
  const [rowSelection, setRowSelection] = useState({});
  const [tableData, setTableData] = useState<KvTableKey[]>([]);
  const [isUpdateSheetOpen, setIsUpdateSheetOpen] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [kvKeyToEdit, setKvKeyToEdit] = useState<KvTableKey | null>(null);
  const [kvKeysToDelete, setKvKeysToDelete] = useState<KvTableKey[]>([]);
  const {
    kvKeys,
    prefix,
    isInitialLoading,
    isLoadingNextKeys,
    hasNextKeys,
    loadNextKeys,
    reloadKeys,
    setKey,
    setPrefix,
    deleteKeys,
  } = useKvKeys(namespace.id);

  const openKvItemUpdateSheet = (key: KvTableKey) => {
    setKvKeyToEdit(key);
    setIsUpdateSheetOpen(true);
  };

  const openKvKeysDeleteDialog = (keys: KvTableKey[]) => {
    setKvKeysToDelete(keys);
    setIsDialogOpen(true);
  };

  const openKvKeysDeleteDialogWithSelectedItems = () => {
    const selectedKeys = table.getSelectedRowModel().rows.map((row) => row.original);
    setKvKeysToDelete(selectedKeys);
    setIsDialogOpen(true);
  };

  const closeKvKeysDeleteDialog = () => {
    setKvKeysToDelete([]);
    setIsDialogOpen(false);
  };

  const refreshKeys = async () => {
    setIsRefreshing(true);

    try {
      await reloadKeys();
      setRowSelection({});
    } catch (e) {
      console.error('Error refreshing keys:', e);
    } finally {
      setIsRefreshing(false);
    }
  };

  const deleteKvItemsAndReload = async () => {
    setIsDeleting(true);

    try {
      await deleteKeys(kvKeysToDelete.map((item) => item.name));
      await reloadKeys();

      const keys = kvKeysToDelete.map((key) => key.name);
      const newSelection: Record<string, unknown> = { ...rowSelection };
      keys.forEach((key) => delete newSelection[key]);
      setRowSelection(newSelection);
    } catch (e) {
      console.error('Error deleting keys:', e);
    } finally {
      setIsDialogOpen(false);
      setIsDeleting(false);
    }
  };

  const changePrefixOnEnter = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      setPrefix(event.currentTarget.value);
      setRowSelection({});
    }
  };

  const changePrefixOnBlur = (event: FocusEvent<HTMLInputElement>) => {
    const newPrefix = event.currentTarget.value;
    const currentPrefix = prefix ?? '';
    if (newPrefix !== currentPrefix) {
      setPrefix(newPrefix);
      setRowSelection({});
    }
  };

  const columns = useMemo<ColumnDef<KvTableKey>[]>(() => {
    return [
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
        id: 'key',
        accessorKey: 'name',
        header: 'Key',
        cell: (cell) => (
          <Button
            onClick={() => openKvItemUpdateSheet(cell.row.original)}
            variant="link"
            className="w-fit h-fit p-0 text-left text-foreground"
          >
            {cell.getValue() as string}
          </Button>
        ),
      },
      {
        id: 'expiration',
        accessorKey: 'expiration',
        header: 'Expiration',
        cell: (cell) => {
          let formattedExpirationDate = '-';
          if (cell.getValue()) {
            formattedExpirationDate = format(cell.getValue() as Date, 'yyyy-MM-dd HH:mm');
          }

          return <>{formattedExpirationDate}</>;
        },
        meta: {
          width: '180px',
        },
      },
      {
        id: 'actions',
        accessorKey: 'actions',
        header: '',
        meta: {
          width: '50px',
        },
        cell: (cell) => (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                className="flex size-8 text-muted-foreground data-[state=open]:bg-muted"
                size="icon"
              >
                <MoreVerticalIcon />
                <span className="sr-only">Open menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-32">
              <DropdownMenuItem onClick={() => openKvItemUpdateSheet(cell.row.original)}>
                <EditIcon />
                Edit
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => openKvKeysDeleteDialog([cell.row.original])}>
                <TrashIcon />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ),
      },
    ];
  }, []);

  const table = useReactTable({
    data: tableData,
    columns,
    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    getRowId: (row) => row.name,
    getCoreRowModel: getCoreRowModel(),
    state: {
      rowSelection,
    },
  });

  const deleteDescription =
    kvKeysToDelete.length === 1
      ? `the item with the key <b>${kvKeysToDelete[0].name}</b>`
      : `<b>${kvKeysToDelete.length}</b> items`;

  useEffect(() => {
    setTableData(kvKeys?.keys.map((key) => ({ ...key, namespaceId: namespace.id })) ?? []);
  }, [kvKeys]);
  useEffect(() => setRowSelection({}), [namespace]);

  const deleteButtonEnabled = table.getIsAllPageRowsSelected() || table.getIsSomePageRowsSelected();
  return (
    <div>
      <div className="w-full grid grid-cols-[1fr_auto] gap-2 align-items-right py-4">
        <div className="relative">
          <Search className="pointer-events-none absolute left-2 top-1/2 size-4 -translate-y-1/2 select-none opacity-50" />
          <Input
            type="search"
            placeholder="Search keys by prefix..."
            className="pl-8 max-w-[250px] h-8"
            disabled={isInitialLoading || isRefreshing}
            onKeyDown={changePrefixOnEnter}
            onBlur={changePrefixOnBlur}
          />
        </div>

        <div className="grid grid-cols-[auto_auto_auto] gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={!deleteButtonEnabled || isRefreshing}
            onClick={openKvKeysDeleteDialogWithSelectedItems}
          >
            <Trash2Icon />
            <span className="hidden lg:inline">Delete</span>
          </Button>
          <KvItemCreateSheet namespaceId={namespace.id} onCreate={async () => await reloadKeys()}>
            <Button variant="outline" size="sm" disabled={isRefreshing}>
              <PlusIcon />
              <span className="hidden lg:inline">Add Item</span>
            </Button>
          </KvItemCreateSheet>
          <Button variant="outline" size="sm" disabled={isRefreshing} onClick={refreshKeys}>
            <RefreshCcwIcon className={isRefreshing ? 'animate-spin' : ''} />
          </Button>
        </div>
      </div>

      <div className="rounded-md border">
        <Table className="table-fixed">
          <KvTableHeader headerGroups={table.getHeaderGroups()} />

          {isInitialLoading || isRefreshing ? (
            <LoadingTableBody pageSize={25} columns={columns} />
          ) : table.getRowModel().rows?.length ? (
            <KvTableBody rows={table.getRowModel().rows} />
          ) : (
            <EmptyTableBody columnsLength={columns.length} />
          )}
        </Table>
      </div>

      {hasNextKeys && !isInitialLoading && !isRefreshing ? (
        <div className="flex items-center justify-center space-x-2 py-4">
          <Button variant="outline" size="sm" onClick={loadNextKeys} disabled={isLoadingNextKeys}>
            {isLoadingNextKeys ? (
              <>
                <LoadingSpinner /> Loading...
              </>
            ) : (
              <>
                <ArrowDown /> Load more
              </>
            )}
          </Button>
        </div>
      ) : null}

      <KvItemUpdateSheet
        namespaceId={namespace.id}
        itemKey={kvKeyToEdit?.name ?? ''}
        itemMetadata={kvKeyToEdit?.metadata ?? null}
        open={isUpdateSheetOpen}
        onUpdate={(kvItem) => setKey({ name: kvItem.key, expiration: kvItem.expiration, metadata: kvItem.metadata })}
        onOpenChange={setIsUpdateSheetOpen}
      />

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent closeDisabled={isDeleting}>
          <DialogHeader>
            <DialogTitle>Are you sure?</DialogTitle>
            <DialogDescription>
              Do you really want to delete <span dangerouslySetInnerHTML={{ __html: deleteDescription }} />? This action
              cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="secondary" disabled={isDeleting} onClick={closeKvKeysDeleteDialog}>
              Cancel
            </Button>
            <Button variant="destructive" disabled={isDeleting} onClick={deleteKvItemsAndReload}>
              {isDeleting ? (
                <>
                  <LoadingSpinner /> Deleting...
                </>
              ) : (
                <>
                  <TrashIcon /> Delete
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

interface LoadingTableBodyProps {
  pageSize?: number;
  columns: ColumnDef<KvTableKey>[];
}

const LoadingTableBody: FunctionComponent<LoadingTableBodyProps> = ({ pageSize = 10, columns }) => {
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
