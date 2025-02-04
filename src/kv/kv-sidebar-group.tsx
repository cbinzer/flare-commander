'use client';

import { ChevronRight, FolderKey } from 'lucide-react';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import {
  SidebarGroup,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@/components/ui/sidebar';
import { useNamespaces } from '@/kv/kv-hooks.ts';
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { FunctionComponent } from 'react';
import { KvNamespace } from '@/kv/kv-models.ts';

export function KvSidebarGroup() {
  const { loading, namespaces, getNamespaces, setNamespaces } = useNamespaces();

  const loadNamespacesOnOpen = async (open: boolean) => {
    if (open) {
      await getNamespaces();
    } else {
      setNamespaces(null);
    }
  };

  return (
    <SidebarGroup>
      <SidebarMenu>
        <Collapsible
          key="KV"
          asChild
          defaultOpen={false}
          className="group/collapsible"
          onOpenChange={loadNamespacesOnOpen}
        >
          <SidebarMenuItem>
            <CollapsibleTrigger asChild>
              <SidebarMenuButton tooltip="KV tooltip">
                <FolderKey />
                <span>KV</span>
                <ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
              </SidebarMenuButton>
            </CollapsibleTrigger>

            <CollapsibleContent>
              {loading || !namespaces ? (
                <KvSidebarMenuSkeleton />
              ) : (
                <KvSidebarMenu namespaces={namespaces} />
              )}
            </CollapsibleContent>
          </SidebarMenuItem>
        </Collapsible>
      </SidebarMenu>
    </SidebarGroup>
  );
}

const KvSidebarMenu: FunctionComponent<{ namespaces: KvNamespace[] }> = ({
  namespaces = [],
}) => {
  return (
    <SidebarMenuSub>
      {namespaces.map((namespace) => (
        <SidebarMenuSubItem key={namespace.id}>
          <SidebarMenuSubButton asChild>
            <a href="#">
              <span>{namespace.title}</span>
            </a>
          </SidebarMenuSubButton>
        </SidebarMenuSubItem>
      ))}
    </SidebarMenuSub>
  );
};

const KvSidebarMenuSkeleton: FunctionComponent = () => {
  return (
    <SidebarMenuSub>
      <SidebarMenuSubItem key="skeleton-1">
        <SidebarMenuSubButton asChild>
          <a href="#">
            <Skeleton className="h-4 w-[120px]" />
          </a>
        </SidebarMenuSubButton>
      </SidebarMenuSubItem>
      <SidebarMenuSubItem key="skeleton-2">
        <SidebarMenuSubButton asChild>
          <a href="#">
            <Skeleton className="h-4 w-[150px]" />
          </a>
        </SidebarMenuSubButton>
      </SidebarMenuSubItem>
      <SidebarMenuSubItem key="skeleton-3">
        <SidebarMenuSubButton asChild>
          <a href="#">
            <Skeleton className="h-4 w-[130px]" />
          </a>
        </SidebarMenuSubButton>
      </SidebarMenuSubItem>
    </SidebarMenuSub>
  );
};
