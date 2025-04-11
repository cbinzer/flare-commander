import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
} from '@/components/ui/sidebar';
import { KvSidebarGroup } from '@/kv/kv-sidebar-group.tsx';
import { Outlet } from 'react-router';
import { GalleryVerticalEnd } from 'lucide-react';

export default function MainPage() {
  return (
    <SidebarProvider>
      <Sidebar collapsible="icon">
        <SidebarHeader>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton size="lg" asChild>
                <div>
                  <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                    <GalleryVerticalEnd className="size-4" />
                  </div>
                  <div className="flex flex-col gap-0.5 leading-none">
                    <span className="font-semibold">FlareCommander</span>
                    <span className="">v0.1.0</span>
                  </div>
                </div>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarHeader>
        <SidebarContent>
          <KvSidebarGroup />
        </SidebarContent>
        <SidebarFooter></SidebarFooter>
      </Sidebar>

      <SidebarInset>
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
}
