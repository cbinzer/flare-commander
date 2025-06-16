import { TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';
import { KvTableKey } from '@/features/kv/kv-models.ts';
import { calcElementWidth } from '@/features/kv/lib/kv-table-utils.ts';
import { flexRender, HeaderGroup } from '@tanstack/react-table';

interface DataTableHeaderProps {
  headerGroups: HeaderGroup<KvTableKey>[];
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
                {header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}
              </TableHead>
            );
          })}
        </TableRow>
      ))}
    </TableHeader>
  );
}
