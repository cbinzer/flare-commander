import { flexRender, HeaderGroup } from '@tanstack/react-table';
import { TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';
import { KvItem } from '@/kv/kv-models.ts';
import { calcElementWidth } from '@/kv/table/kv-table-utils.ts';

interface DataTableHeaderProps {
  headerGroups: HeaderGroup<KvItem>[];
}

export function KvTableHeader(props: DataTableHeaderProps) {
  return (
    <TableHeader>
      {props.headerGroups.map((headerGroup) => (
        <TableRow key={headerGroup.id} className="h-[40px]">
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
