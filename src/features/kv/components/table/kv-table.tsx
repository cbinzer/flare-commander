import { Button } from '@/components/ui/button.tsx';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { Table, TableBody, TableCell, TableRow } from '@/components/ui/table.tsx';
import { KvNamespace, KvTableKey } from '@/features/kv/kv-models.ts';
import { KvTableBody } from '@/features/kv/components/table/kv-table-body.tsx';
import { KvTableHeader } from '@/features/kv/components/table/kv-table-header.tsx';
import { ColumnDef, getCoreRowModel, useReactTable } from '@tanstack/react-table';
import { format } from 'date-fns';
import { FocusEvent, FunctionComponent, KeyboardEvent, MouseEventHandler, useEffect, useMemo, useState } from 'react';
import KvPairUpdateSheet from '../kv-pair-update-sheet.tsx';
import {
  ArrowDown,
  ChevronDown,
  DownloadIcon,
  EditIcon,
  MoreVerticalIcon,
  PlusIcon,
  RefreshCcwIcon,
  Search,
  Trash2Icon,
  TrashIcon,
  UploadIcon,
} from 'lucide-react';
import KvPairCreateSheet from '@/features/kv/components/kv-pair-create-sheet.tsx';
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
import { useKvKeys } from '@/features/kv/hooks/use-kv-keys.ts';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeFile } from '@tauri-apps/plugin-fs';
import { useKvPairs } from '@/features/kv/hooks/use-kv-pairs.ts';
import { toast } from 'sonner';
import { useKvPair } from '@/features/kv/hooks/use-kv-pair.ts';

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
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [importFileName, setImportFileName] = useState('');
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
    setPrefix,
    deleteKeys,
  } = useKvKeys(namespace.id);
  const { createKvPairJSONExport, createKvValueExport } = useKvPair();
  const { createKvPairsJSONExport } = useKvPairs();

  const openKvPairUpdateSheet = (key: KvTableKey) => {
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

  const exportKvPairs = async () => {
    const path = await save({ filters: [{ name: 'JSON', extensions: ['json'] }] });
    if (path) {
      setIsExporting(true);

      toast.promise(
        async () => {
          const selectedKeys = table
            .getSelectedRowModel()
            .rows.map((row) => row.original)
            .map((tableKey) => tableKey.name);
          const kvPairsExport = await createKvPairsJSONExport(namespace.id, selectedKeys);
          await writeFile(path, kvPairsExport);
        },
        {
          position: 'top-center',
          loading: 'Exporting KV Pairs...',
          success: () => `KV Pairs successfully exported!`,
          error: (error) => `Error exporting KV Pairs: ${error.message}`,
          finally: () => setIsExporting(false),
        },
      );
    }
  };

  const exportKvPair = async (key: string) => {
    const path = await save({ filters: [{ name: 'JSON', extensions: ['json'] }] });
    if (path) {
      setIsExporting(true);

      toast.promise(
        async () => {
          const kvPairsExport = await createKvPairJSONExport(namespace.id, key);
          await writeFile(path, kvPairsExport);
        },
        {
          position: 'top-center',
          loading: `Exporting KV Pair with key '${key}'...`,
          success: () => `KV Pair with key '${key}' successfully exported!`,
          error: (error) => `Error exporting KV Pair with key '${key}': ${error.message}`,
          finally: () => setIsExporting(false),
        },
      );
    }
  };

  const exportKvValue = async (key: string) => {
    const path = await save();
    if (path) {
      setIsExporting(true);

      toast.promise(
        async () => {
          const kvValueExport = await createKvValueExport(namespace.id, key);
          await writeFile(path, kvValueExport);
        },
        {
          position: 'top-center',
          loading: `Exporting KV Value from key '${key}'...`,
          success: () => `KV Value from key '${key}' successfully exported!`,
          error: (error) => `Error exporting KV Pair from key '${key}': ${error.message}`,
          finally: () => setIsExporting(false),
        },
      );
    }
  };

  const importKvPairsFromJSON = async () => {
    const path = await open({ filters: [{ name: 'JSON', extensions: ['json'] }] });
    if (path) {
      setIsImporting(true);
      setImportFileName(path.split('/').pop() || '');

      const kvPairsImport = await readTextFile(path);
      const kvPairsJSONImport = JSON.parse(kvPairsImport);
      console.log(kvPairsJSONImport);

      setTimeout(() => setIsImporting(false), 5000);
    }
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

  const deleteKvPairAndReload = async () => {
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
            onClick={() => openKvPairUpdateSheet(cell.row.original)}
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
            <DropdownMenuContent align="end" className="w-36">
              <DropdownMenuItem onClick={() => openKvPairUpdateSheet(cell.row.original)}>
                <EditIcon />
                Edit
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => openKvKeysDeleteDialog([cell.row.original])}>
                <TrashIcon />
                Delete
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => exportKvPair(cell.row.original.name)} disabled={isExporting}>
                <DownloadIcon />
                Export
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => exportKvValue(cell.row.original.name)} disabled={isExporting}>
                <DownloadIcon />
                Export Value
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        ),
      },
    ];
  }, [isExporting]);

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

  const actionButtonsEnabled = table.getIsAllPageRowsSelected() || table.getIsSomePageRowsSelected();
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

        <div className="grid grid-cols-[auto_auto_auto_auto] gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={!actionButtonsEnabled || isRefreshing || isExporting}
            onClick={openKvKeysDeleteDialogWithSelectedItems}
          >
            <Trash2Icon />
            <span className="hidden lg:inline">Delete</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={!actionButtonsEnabled || isRefreshing || isExporting}
            onClick={exportKvPairs}
          >
            <DownloadIcon />
            <span className="hidden lg:inline">Export</span>
          </Button>
          <KvPairCreateSheet namespaceId={namespace.id} onCreate={async () => await reloadKeys()}>
            <AddButton disabled={isRefreshing} onClickImport={importKvPairsFromJSON} />
          </KvPairCreateSheet>
          <Button variant="outline" size="sm" disabled={isRefreshing} onClick={refreshKeys}>
            <RefreshCcwIcon className={isRefreshing ? 'animate-spin' : ''} />
          </Button>
        </div>
      </div>

      <div className="rounded-md border">
        <Table className="table-fixed">
          <KvTableHeader headerGroups={table.getHeaderGroups()} />

          {isInitialLoading || isRefreshing ? (
            <LoadingTableBody pageSize={10} columns={columns} />
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

      <KvPairUpdateSheet
        namespaceId={namespace.id}
        itemKey={kvKeyToEdit?.name ?? ''}
        open={isUpdateSheetOpen}
        onUpdate={reloadKeys}
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
            <Button variant="destructive" disabled={isDeleting} onClick={deleteKvPairAndReload}>
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
      <KvPairsImportDialog open={isImporting} fileName={importFileName} />
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
          <TableRow key={i} className={'h-[54px]'} selectable={false}>
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

const AddButton: FunctionComponent<{
  disabled?: boolean;
  onClick?: MouseEventHandler<HTMLButtonElement> | undefined;
  onClickImport?: MouseEventHandler<HTMLDivElement> | undefined;
}> = ({ disabled, onClick, onClickImport }) => {
  return (
    <div className="flex">
      <Button variant="outline" size="sm" disabled={disabled} className="rounded-r-none" onClick={onClick}>
        <PlusIcon />
        <span className="hidden lg:inline">Add Pair</span>
      </Button>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="sm" className={'rounded-l-none border-l-0 px-2'} disabled={disabled}>
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem className="h-8 rounded-md px-3 text-xs" onClick={onClickImport} disabled={disabled}>
            <UploadIcon />
            Import from JSON
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

const KvPairsImportDialog: FunctionComponent<{ open?: boolean; fileName?: string }> = ({ open, fileName = '' }) => {
  return (
    <Dialog open={open}>
      <DialogContent closeDisabled={true} closeVisible={false}>
        <div className="grid grid-cols-[auto_1fr] items-center gap-2">
          <LoadingSpinner className="inline h-[18px] w-[18px]" />
          <span>
            Importing KV Pairs from <b>{fileName}</b>...
          </span>
        </div>
      </DialogContent>
    </Dialog>
  );
};
