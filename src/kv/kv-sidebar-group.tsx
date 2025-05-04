'use client';

import { EditIcon, FolderKey, MoreHorizontal, PlusIcon, RefreshCcwIcon, TrashIcon } from 'lucide-react';
import {
  SidebarGroup,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubAction,
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useIsMobile } from '@/hooks/use-mobile.tsx';
import KvNamespaceCreateSheet from '@/kv/kv-namespace-create-sheet.tsx';

export function KvSidebarGroup() {
  const [activeNamespaceId, setActiveNamespaceId] = useState<string | undefined>();
  const [isReloading, setIsReloading] = useState(false);
  const [isCreateSheetOpen, setIsCreateSheetOpen] = useState(false);
  const { isListing, namespaces, listNamespaces, relistNamespaces } = useNamespaces();
  const { handleError } = useError();
  const isMobile = useIsMobile();
  const isLoading = isListing || isReloading;

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
      await listNamespaces();
    } catch (error) {
      handleError(error as Error);
    }
  };

  const reloadNamespaces = async () => {
    setIsReloading(true);
    await relistNamespaces();
    setIsReloading(false);
  };

  useEffect(() => {
    loadNamespaces().then();
  }, []);

  return (
    <>
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
            <SidebarMenuButton tooltip="KV tooltip" unselectable="on">
              <FolderKey />
              <span>KV</span>
              {/*<ChevronRight className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />*/}
            </SidebarMenuButton>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuAction showOnHover={!isReloading} disabled={isLoading}>
                  {isReloading ? <RefreshCcwIcon className="animate-spin" /> : <MoreHorizontal />}
                  <span className="sr-only">More</span>
                </SidebarMenuAction>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                className="w-48 rounded-lg"
                side={isMobile ? 'bottom' : 'right'}
                align={isMobile ? 'end' : 'start'}
              >
                <DropdownMenuItem onClick={reloadNamespaces}>
                  <RefreshCcwIcon />
                  <span>Reload</span>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => setIsCreateSheetOpen(true)}>
                  <PlusIcon />
                  <span>Add Namespace</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
            {/*</CollapsibleTrigger>*/}

            {/*<CollapsibleContent>*/}
            {isLoading || !namespaces ? (
              <KvSidebarMenuSkeleton />
            ) : (
              <KvSidebarMenu
                namespaces={namespaces}
                activeNamespaceId={activeNamespaceId}
                onSelectNamespace={(namespace) => setActiveNamespaceId(namespace.id)}
              />
            )}
            {/*</CollapsibleContent>*/}
          </SidebarMenuItem>
          {/*</Collapsible>*/}
        </SidebarMenu>
      </SidebarGroup>
      <KvNamespaceCreateSheet
        open={isCreateSheetOpen}
        onOpenChange={setIsCreateSheetOpen}
        onCreate={relistNamespaces}
      />
    </>
  );
}

interface KvSidebarMenuProps {
  namespaces: KvNamespace[];
  activeNamespaceId?: String;
  onSelectNamespace?: (namespace: KvNamespace) => void;
}

const KvSidebarMenu: FunctionComponent<KvSidebarMenuProps> = ({
  namespaces = [],
  activeNamespaceId,
  onSelectNamespace = () => {},
}) => {
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const [activeNamespace, setActiveNamespace] = useState<KvNamespace | undefined>();

  useEffect(() => {
    setActiveNamespace(namespaces.find((namespace) => namespace.id === activeNamespaceId));
  }, [activeNamespaceId]);

  const openKvSection = (event: MouseEvent<HTMLAnchorElement>, namespace: KvNamespace) => {
    event.preventDefault();
    setActiveNamespace(namespace);
    onSelectNamespace(namespace);
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
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <SidebarMenuSubAction showOnHover>
                      <MoreHorizontal />
                      <span className="sr-only">More</span>
                    </SidebarMenuSubAction>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent
                    className="w-48 rounded-lg"
                    side={isMobile ? 'bottom' : 'right'}
                    align={isMobile ? 'end' : 'start'}
                  >
                    <DropdownMenuItem>
                      <EditIcon />
                      <span>Edit</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                      <TrashIcon />
                      <span>Delete</span>
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
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
