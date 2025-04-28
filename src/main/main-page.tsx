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
import { AccountSidebarMenu } from '@/common/account-sidebar-menu.tsx';
import logoUrl from '../assets/logo.svg';

export default function MainPage() {
  return (
    <SidebarProvider>
      <Sidebar collapsible="icon">
        <SidebarHeader>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton size="lg" asChild>
                <div>
                  <div className="flex aspect-square size-10 items-center justify-center rounded-lg text-sidebar-primary-foreground">
                    <img src={logoUrl} alt="FlareCommander logo" />
                  </div>
                  <div className="flex flex-col gap-0.5 leading-none">
                    <span className="font-semibold">FlareCommander</span>
                    <span>v0.1.0</span>
                  </div>
                </div>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarHeader>
        <SidebarContent>
          <KvSidebarGroup />
        </SidebarContent>
        <SidebarFooter>
          <AccountSidebarMenu />
        </SidebarFooter>
      </Sidebar>

      <SidebarInset>
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
}
