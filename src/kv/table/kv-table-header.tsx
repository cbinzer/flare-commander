import { flexRender, HeaderGroup } from '@tanstack/react-table';
import { TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';
import { KvKey } from '@/kv/kv-models.ts';
import { calcElementWidth } from '@/kv/table/kv-table-utils.ts';

interface DataTableHeaderProps {
  headerGroups: HeaderGroup<KvKey>[];
}

export function KvTableHeader(props: DataTableHeaderProps) {
  return (
    <TableHeader>
      {props.headerGroups.map((headerGroup) => (
        <TableRow key={headerGroup.id} className="h-[40px]" selectable={false}>
          {headerGroup.headers.map((header) => {
            return (
              <TableHead
                key={header.id}
                style={{
                  width: calcElementWidth(header),
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
  );
}
