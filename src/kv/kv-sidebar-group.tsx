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
import { FunctionComponent, MouseEvent, useState } from 'react';
import { KvNamespace } from '@/kv/kv-models.ts';
import { useError } from '@/common/common-hooks.ts';
import { useNavigate } from 'react-router';

export function KvSidebarGroup() {
  const { loading, namespaces, getNamespaces, setNamespaces } = useNamespaces();
  const { handleError } = useError();

  const loadNamespacesOnOpen = async (open: boolean) => {
    if (!open) {
      setNamespaces(null);
      return;
    }

    try {
      await getNamespaces();
    } catch (error) {
      handleError(error as Error);
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
  const navigate = useNavigate();
  const [activeNamespace, setActiveNamespace] = useState<KvNamespace | null>(
    null,
  );

  const openKvSection = (
    event: MouseEvent<HTMLAnchorElement>,
    namespace: KvNamespace,
  ) => {
    event.preventDefault();
    setActiveNamespace(namespace);
    navigate(`namespaces/${namespace.id}`, { state: namespace });
  };

  return (
    <SidebarMenuSub>
      {namespaces.map((namespace) => (
        <SidebarMenuSubItem key={namespace.id}>
          <SidebarMenuSubButton
            asChild
            isActive={activeNamespace?.id === namespace.id}
          >
            <a href="#" onClick={(event) => openKvSection(event, namespace)}>
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
