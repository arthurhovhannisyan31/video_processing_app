import type { PropsWithChildren } from "react";

import { InteractiveDotGrid } from "components/aicanvas/dot-grid";
import { SiteHeader } from "components/site-header";
import { SidebarInset, SidebarProvider } from "components/ui/sidebar";

export default function AppLayout({ children }: PropsWithChildren) {
  return (
    <SidebarProvider
      style={
        {
          "--sidebar-width": "calc(var(--spacing) * 72)",
          "--header-height": "calc(var(--spacing) * 12)",
        } as React.CSSProperties
      }
    >
      <SidebarInset>
        <InteractiveDotGrid>
          <div
            className={
              "flex flex-col h-full absolute top-0 left-0 right-0 bottom-0 z-10"
            }
          >
            <SiteHeader />
            <div className="flex flex-1 flex-col overflow-auto">
              <div className="@container/main flex flex-1 flex-col gap-2">
                <div className="flex flex-col gap-4 py-4 md:gap-6 md:py-6 h-full">
                  {children}
                </div>
              </div>
            </div>
          </div>
        </InteractiveDotGrid>
      </SidebarInset>
    </SidebarProvider>
  );
}
