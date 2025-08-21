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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu.tsx';
import { useIsMobile } from '@/hooks/use-mobile.ts';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { useBuckets } from '@/features/r2/use-buckets.ts';
import { Bucket } from '@/features/r2/r2-models.ts';
import SidebarMenuSkeleton from '@/components/ui/sidebar-menu-skeleton.tsx';

export function R2SidebarMenu() {
  const [isOpen, setIsOpen] = useState(false);
  const { buckets, loading, reloading, loadingNext, hasNext, loadBuckets, reloadBuckets, loadNextBuckets } =
    useBuckets();
  // const { handleError } = useError();
  const isMobile = useIsMobile();

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
              <SidebarMenuAction showOnHover={!reloading} disabled={loading}>
                {reloading ? <RefreshCcwIcon className="animate-spin" /> : <MoreHorizontal />}
                <span className="sr-only">More</span>
              </SidebarMenuAction>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="w-48 rounded-lg"
              side={isMobile ? 'bottom' : 'right'}
              align={isMobile ? 'end' : 'start'}
            >
              <DropdownMenuItem disabled={!isOpen} onClick={reloadBuckets}>
                <RefreshCcwIcon />
                <span>Reload</span>
              </DropdownMenuItem>
              {/*<DropdownMenuItem disabled={true}>*/}
              {/*  <PlusIcon />*/}
              {/*  <span>Add Bucket</span>*/}
              {/*</DropdownMenuItem>*/}
            </DropdownMenuContent>
          </DropdownMenu>

          <CollapsibleContent>
            {loading && !loadingNext ? (
              <SidebarMenuSkeleton />
            ) : (
              <R2SidebarMenuSub
                buckets={buckets}
                hasNext={hasNext}
                loadingNext={loadingNext}
                loadNextBuckets={loadNextBuckets}
              />
            )}
          </CollapsibleContent>
        </SidebarMenuItem>
      </Collapsible>
    </SidebarMenu>
  );
}

interface R2SidebarMenuProps {
  buckets: Bucket[];
  hasNext: boolean;
  loadingNext: boolean;
  loadNextBuckets: () => Promise<void>;
}

const R2SidebarMenuSub: FunctionComponent<R2SidebarMenuProps> = ({
  buckets,
  hasNext,
  loadingNext,
  loadNextBuckets,
}) => {
  const [activeBucket, setActiveBucket] = useState<Bucket | undefined>();
  // const navigate = useNavigate();
  const isMobile = useIsMobile();

  const openR2Section = (event: MouseEvent<HTMLAnchorElement>, bucket: Bucket) => {
    event.preventDefault();
    setActiveBucket(bucket);
    // navigate(`namespaces/${namespace.id}`, { state: namespace });
  };

  return (
    <SidebarMenuSub>
      {buckets.map((bucket) => (
        <SidebarMenuSubItem>
          <SidebarMenuSubButton asChild isActive={activeBucket?.name === bucket.name}>
            <a href="#" onClick={(event) => openR2Section(event, bucket)}>
              <span>{bucket.name}</span>
            </a>
          </SidebarMenuSubButton>
          <DropdownMenu>
            <DropdownMenuTrigger asChild hidden={true} disabled={true}>
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
      ))}
      {hasNext && (
        <SidebarMenuSubItem>
          <SidebarMenuButton
            className="text-sidebar-foreground/55 cursor-pointer"
            disabled={loadingNext}
            onClick={loadNextBuckets}
          >
            {loadingNext ? (
              <>
                <Loader2Icon className="animate-spin" /> Loading...
              </>
            ) : (
              <>
                <ArrowDown />
                <span>Load More</span>
              </>
            )}
          </SidebarMenuButton>
        </SidebarMenuSubItem>
      )}
    </SidebarMenuSub>
  );
};
