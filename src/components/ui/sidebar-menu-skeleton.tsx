import { FunctionComponent } from 'react';
import { SidebarMenuSub, SidebarMenuSubItem } from '@/components/ui/sidebar.tsx';
import { Skeleton } from '@/components/ui/skeleton.tsx';

const SidebarMenuSkeleton: FunctionComponent = () => {
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

export default SidebarMenuSkeleton;
