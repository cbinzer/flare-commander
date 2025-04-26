'use client';

import { FolderKey, RefreshCcwIcon } from 'lucide-react';
import {
  SidebarGroup,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@/components/ui/sidebar';
import { useNamespaces } from '@/kv/kv-hooks.ts';
import { Skeleton } from '@/components/ui/skeleton.tsx';
import { FunctionComponent, MouseEvent, useEffect, useState } from 'react';
import { KvNamespace } from '@/kv/kv-models.ts';
import { useError } from '@/common/common-hooks.ts';
import { useNavigate } from 'react-router';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

export function KvSidebarGroup() {
  const [isRefreshing, setIsRefreshing] = useState(false);
  const { loading, namespaces, getNamespaces } = useNamespaces();
  const { handleError } = useError();

  // const loadNamespacesOnOpen = async (open: boolean) => {
  //   if (!open) {
  //     setNamespaces(null);
  //     return;
  //   }
  //
  //   await loadNamespaces();
  // };

  const loadNamespaces = async () => {
    try {
      await getNamespaces();
    } catch (error) {
      handleError(error as Error);
    }
  };

  const refreshNamespaces = async () => {
    setIsRefreshing(true);
    await loadNamespaces();
    setIsRefreshing(false);
  };

  useEffect(() => {
    loadNamespaces().then();
  }, []);

  return (
    <SidebarGroup>
      <SidebarMenu>
        {/*<Collapsible*/}
        {/*  key="KV"*/}
        {/*  asChild*/}
        {/*  defaultOpen={false}*/}
        {/*  className="group/collapsible"*/}
        {/*  onOpenChange={loadNamespacesOnOpen}*/}
        {/*>*/}
        <SidebarMenuItem>
          {/*<CollapsibleTrigger asChild>*/}
          <SidebarMenuButton tooltip="KV tooltip" unselectable="on" className="">
            <FolderKey />
            <span>KV</span>
            {/*<ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />*/}
          </SidebarMenuButton>
          <SidebarMenuAction
            onClick={refreshNamespaces}
            showOnHover={!isRefreshing}
            disabled={loading}
            className="rounded-sm data-[state=open]:bg-accent"
          >
            <RefreshCcwIcon className={isRefreshing ? 'animate-spin' : ''} />
            <span className="sr-only">More</span>
          </SidebarMenuAction>
          {/*</CollapsibleTrigger>*/}

          {/*<CollapsibleContent>*/}
          {loading || !namespaces ? <KvSidebarMenuSkeleton /> : <KvSidebarMenu namespaces={namespaces} />}
          {/*</CollapsibleContent>*/}
        </SidebarMenuItem>
        {/*</Collapsible>*/}
      </SidebarMenu>
    </SidebarGroup>
  );
}

const KvSidebarMenu: FunctionComponent<{ namespaces: KvNamespace[] }> = ({ namespaces = [] }) => {
  const navigate = useNavigate();
  const [activeNamespace, setActiveNamespace] = useState<KvNamespace | null>(null);

  const openKvSection = (event: MouseEvent<HTMLAnchorElement>, namespace: KvNamespace) => {
    event.preventDefault();
    setActiveNamespace(namespace);
    navigate(`namespaces/${namespace.id}`, { state: namespace });
  };

  return (
    <SidebarMenuSub>
      {namespaces.map((namespace) => (
        <TooltipProvider delayDuration={1000} key={namespace.id}>
          <Tooltip>
            <TooltipTrigger asChild>
              <SidebarMenuSubItem>
                <SidebarMenuSubButton asChild isActive={activeNamespace?.id === namespace.id}>
                  <a href="#" onClick={(event) => openKvSection(event, namespace)}>
                    <span>{namespace.title}</span>
                  </a>
                </SidebarMenuSubButton>
              </SidebarMenuSubItem>
            </TooltipTrigger>
            <TooltipContent>
              <p>{namespace.title}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      ))}
    </SidebarMenuSub>
  );
};

const KvSidebarMenuSkeleton: FunctionComponent = () => {
  return (
    <SidebarMenuSub>
      <SidebarMenuSubItem key="skeleton-1">
        <Skeleton className="ml-2 my-1 h-4 w-[120px]" />
      </SidebarMenuSubItem>
      <SidebarMenuSubItem key="skeleton-2">
        <Skeleton className="ml-2 my-1 h-4 w-[150px]" />
      </SidebarMenuSubItem>
      <SidebarMenuSubItem key="skeleton-3">
        <Skeleton className="ml-2 my-1 h-4 w-[130px]" />
      </SidebarMenuSubItem>
    </SidebarMenuSub>
  );
};
