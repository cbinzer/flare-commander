'use client';

import {
  ArrowDown,
  ChevronRight,
  EditIcon,
  HardDrive,
  Loader2Icon,
  MoreHorizontal,
  RefreshCcwIcon,
  TrashIcon,
} from 'lucide-react';
import {
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubAction,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@/components/ui/sidebar.tsx';
import { FunctionComponent, MouseEvent, useState } from 'react';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu.tsx';
import { useIsMobile } from '@/hooks/use-mobile.ts';
import { useError } from '@/hooks/use-error.tsx';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { useBuckets } from '@/features/r2/use-buckets.ts';
import { Bucket } from '@/features/r2/r2-models.ts';
import SidebarMenuSkeleton from '@/components/ui/sidebar-menu-skeleton.tsx';

export function R2SidebarMenu() {
  // const [activeNamespaceId, setActiveNamespaceId] = useState<string | undefined>();
  const [isReloading, setIsReloading] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const { buckets, loadBuckets, loading } = useBuckets();
  const { handleError } = useError();
  const isMobile = useIsMobile();

  const isLoading = loading || isReloading;
  const isLoadMoreVisible = false;

  const loadBucketsOnOpen = async (open: boolean) => {
    setIsOpen(open);

    if (!open) {
      return;
    }

    await loadBuckets();
  };

  return (
    <SidebarMenu>
      <Collapsible key="R2" asChild defaultOpen={false} className="group/collapsible" onOpenChange={loadBucketsOnOpen}>
        <SidebarMenuItem>
          <SidebarMenuButton>
            <HardDrive />
            <span>R2</span>
          </SidebarMenuButton>

          <CollapsibleTrigger asChild>
            <SidebarMenuAction
              className="bg-sidebar-accent text-sidebar-accent-foreground left-2 data-[state=open]:rotate-90"
              showOnHover
            >
              <ChevronRight />
            </SidebarMenuAction>
          </CollapsibleTrigger>

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
              {/*<DropdownMenuItem onClick={reloadNamespaces} disabled={!isOpen}>*/}
              {/*  <RefreshCcwIcon />*/}
              {/*  <span>Reload</span>*/}
              {/*</DropdownMenuItem>*/}
              {/*<DropdownMenuItem onClick={() => setIsCreateSheetOpen(true)}>*/}
              {/*  <PlusIcon />*/}
              {/*  <span>Add Namespace</span>*/}
              {/*</DropdownMenuItem>*/}
            </DropdownMenuContent>
          </DropdownMenu>

          <CollapsibleContent>
            {isLoading ? <SidebarMenuSkeleton /> : <R2SidebarMenuSub buckets={buckets} />}
          </CollapsibleContent>
        </SidebarMenuItem>
      </Collapsible>

      {isLoadMoreVisible && (
        <SidebarMenuItem>
          <SidebarMenuButton
            className="text-sidebar-foreground/70"
            // onClick={loadNextNamespaces}
            disabled={false}
          >
            {false ? (
              <>
                <Loader2Icon className="animate-spin" /> Loading...
              </>
            ) : (
              <>
                <ArrowDown />
                <span>Load more</span>
              </>
            )}
          </SidebarMenuButton>
        </SidebarMenuItem>
      )}
    </SidebarMenu>
  );
}

interface R2SidebarMenuProps {
  buckets: Bucket[];
}

const R2SidebarMenuSub: FunctionComponent<R2SidebarMenuProps> = ({ buckets }) => {
  // const navigate = useNavigate();
  const isMobile = useIsMobile();

  const [activeBucket, setActiveBucket] = useState<Bucket | undefined>();

  const openR2Section = (event: MouseEvent<HTMLAnchorElement>, bucket: Bucket) => {
    event.preventDefault();
    console.log(bucket);
    // navigate(`namespaces/${namespace.id}`, { state: namespace });
  };

  return (
    <SidebarMenuSub>
      {buckets.map((bucket) => (
        <TooltipProvider delayDuration={1000} key={bucket.name}>
          <Tooltip delayDuration={2000}>
            <TooltipTrigger asChild>
              <SidebarMenuSubItem>
                <SidebarMenuSubButton asChild isActive={activeBucket?.name === bucket.name}>
                  <a href="#" onClick={(event) => openR2Section(event, bucket)}>
                    <span>{bucket.name}</span>
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
              <p>{bucket.name}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      ))}
    </SidebarMenuSub>
  );
};
