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
import { KvNamespace, KvNamespaces } from '@/kv/kv-models.ts';
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
import KvNamespaceUpdateSheet from '@/kv/kv-namespace-update-sheet.tsx';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog.tsx';
import { Button } from '@/components/ui/button.tsx';
import { LoadingSpinner } from '@/components/ui/loading-spinner.tsx';

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
                onNamespaceChanged={relistNamespaces}
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
  namespaces: KvNamespaces;
  activeNamespaceId?: String;
  onSelectNamespace?: (namespace: KvNamespace) => void;
  onNamespaceChanged?: (namespace: KvNamespace) => Promise<void>;
}

const KvSidebarMenu: FunctionComponent<KvSidebarMenuProps> = ({
  namespaces,
  activeNamespaceId,
  onSelectNamespace = () => {},
  onNamespaceChanged = () => Promise.resolve(),
}) => {
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  const [activeNamespace, setActiveNamespace] = useState<KvNamespace | undefined>();
  const [isUpdateSheetOpen, setIsUpdateSheetOpen] = useState<boolean>(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState<boolean>(false);
  const [namespaceIdToUpdate, setNamespaceIdToUpdate] = useState<string | undefined>(undefined);
  const [namespaceToDelete, setNamespaceToDelete] = useState<KvNamespace | undefined>(undefined);

  useEffect(() => {
    setActiveNamespace(namespaces.items.find((namespace) => namespace.id === activeNamespaceId));
  }, [activeNamespaceId]);

  const openKvSection = (event: MouseEvent<HTMLAnchorElement>, namespace: KvNamespace) => {
    event.preventDefault();
    setActiveNamespace(namespace);
    onSelectNamespace(namespace);
    navigate(`namespaces/${namespace.id}`, { state: namespace });
  };

  const openUpdateSheet = (namespaceId: string) => {
    setNamespaceIdToUpdate(namespaceId);
    setIsUpdateSheetOpen(true);
  };

  const openDeleteDialog = (namespace: KvNamespace) => {
    setNamespaceToDelete(namespace);
    setIsDeleteDialogOpen(true);
  };

  return (
    <SidebarMenuSub>
      {namespaces.items.map((namespace) => (
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
                    </SidebarMenuSubAction>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent
                    className="w-48 rounded-lg"
                    side={isMobile ? 'bottom' : 'right'}
                    align={isMobile ? 'end' : 'start'}
                  >
                    <DropdownMenuItem onClick={() => openUpdateSheet(namespace.id)}>
                      <EditIcon />
                      <span>Edit</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => openDeleteDialog(namespace)}>
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
      <KvNamespaceUpdateSheet
        namespaceId={namespaceIdToUpdate ?? ''}
        open={isUpdateSheetOpen}
        onOpenChange={setIsUpdateSheetOpen}
        onUpdate={onNamespaceChanged}
      />
      {namespaceToDelete && (
        <KvNamespaceDeleteDialog
          namespace={namespaceToDelete}
          open={isDeleteDialogOpen}
          onDelete={() => onNamespaceChanged(namespaceToDelete)}
          onOpenChange={setIsDeleteDialogOpen}
        />
      )}
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

interface KvNamespaceDeleteDialogProps {
  namespace: KvNamespace;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  onDelete?: () => Promise<void>;
}

const KvNamespaceDeleteDialog: FunctionComponent<KvNamespaceDeleteDialogProps> = ({
  namespace,
  open = false,
  onOpenChange = () => {},
  onDelete = () => Promise.resolve(),
}) => {
  const { deleteNamespace } = useNamespaces();

  const [isDialogOpen, setIsDialogOpen] = useState(open);
  const [isDeleting, setIsDeleting] = useState(false);

  const handleOnOpenChange = (open: boolean) => {
    setIsDialogOpen(open);
    onOpenChange(open);
  };

  const closeDialog = () => handleOnOpenChange(false);

  const deleteNamespaceAndCloseDialog = async () => {
    setIsDeleting(true);

    try {
      await deleteNamespace(namespace.id);
      await onDelete();
      handleOnOpenChange(false);
    } finally {
      setIsDeleting(false);
    }
  };

  useEffect(() => {
    setIsDialogOpen(open);
  }, [open]);

  return (
    <Dialog open={isDialogOpen} onOpenChange={handleOnOpenChange}>
      <DialogContent closeDisabled={isDeleting}>
        <DialogHeader>
          <DialogTitle>Are you sure?</DialogTitle>
          <DialogDescription>
            Do you really want to delete the namespace <b>{namespace.title}</b>? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="secondary" disabled={isDeleting} onClick={closeDialog}>
            Cancel
          </Button>
          <Button variant="destructive" disabled={isDeleting} onClick={deleteNamespaceAndCloseDialog}>
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
  );
};
