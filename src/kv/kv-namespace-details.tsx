import { FunctionComponent, useEffect, useState } from 'react';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb.tsx';
import { Location, useLocation, useNavigate } from 'react-router';
import { KvNamespace } from '@/kv/kv-models.ts';
import { ColumnDef } from '@tanstack/react-table';
import { DataTable } from '@/components/ui/data-table.tsx';
import { Checkbox } from '@/components/ui/checkbox';
import { useKvItems } from '@/kv/kv-hooks.ts';

export const columns: ColumnDef<{ key: string; value: string }>[] = [
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
      width: '30px',
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
];

const KvNamespaceDetails: FunctionComponent = () => {
  const location: Location<KvNamespace> = useLocation();
  const namespace = location.state;
  const navigate = useNavigate();

  if (!namespace) {
    navigate('/');
    return;
  }

  const [currentPage, setCurrentPage] = useState(0);
  const { kvItems, isLoading, hasNextItems, loadPreviousItems, loadNextItems } =
    useKvItems(namespace.id);

  useEffect(() => {
    setCurrentPage(0);
    console.log('loaddata', namespace);
  }, [namespace]);

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>KV</BreadcrumbPage>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage>{namespace.title}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="flex flex-1 flex-col gap-4 p-4">
        <h2 className="scroll-m-20 pb-2 text-3xl font-semibold tracking-tight first:mt-0">
          {namespace.title}
        </h2>

        <DataTable
          columns={columns}
          data={kvItems?.items ?? []}
          onPaginationChange={async (pagination) => {
            if (pagination.pageIndex > currentPage) {
              await loadNextItems();
            } else {
              await loadPreviousItems();
            }

            setCurrentPage(pagination.pageIndex);
          }}
          hasNextPage={hasNextItems}
          isLoading={isLoading}
        />
      </div>
    </>
  );
};

export default KvNamespaceDetails;
