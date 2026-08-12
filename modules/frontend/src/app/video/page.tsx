"use client";

import { useState } from "react";

import { AppSidebar } from "components/app-sidebar";
import { SiteHeader } from "components/site-header";
import { SidebarInset, SidebarProvider } from "components/ui/sidebar";
import { VideoCompress } from "components/video-compress";
import { VideoDropZone } from "components/video-drop-zone";
import { VideoInspectResult } from "components/video-inspect-result";
import { inspectVideo } from "generated/client/sdk.gen";

export default function VideoPage() {
  const [file, setFile] = useState<File | null>(null);
  const [inspectData, setInspectData] = useState<Record<
    string,
    unknown
  > | null>(null);
  const [isInspecting, setIsInspecting] = useState(false);
  const [inspectError, setInspectError] = useState<string | null>(null);

  async function handleFile(f: File) {
    setFile(f);
    setInspectData(null);
    setInspectError(null);
    setIsInspecting(true);
    try {
      const res = await inspectVideo({ body: { file: f } });
      setInspectData(res.data as Record<string, unknown>);
    } catch (err) {
      setInspectError(
        err instanceof Error ? err.message : "Inspection failed.",
      );
    } finally {
      setIsInspecting(false);
    }
  }

  function handleReset() {
    setFile(null);
    setInspectData(null);
    setInspectError(null);
  }

  return (
    <SidebarProvider
      style={
        {
          "--sidebar-width": "calc(var(--spacing) * 72)",
          "--header-height": "calc(var(--spacing) * 12)",
        } as React.CSSProperties
      }
    >
      <AppSidebar variant="inset" />
      <SidebarInset>
        <SiteHeader />
        <div className="flex flex-1 flex-col p-4 md:p-6 gap-6">
          <VideoDropZone
            file={file}
            onFile={handleFile}
            onReset={handleReset}
            disabled={isInspecting}
          />

          {(isInspecting || inspectData || inspectError) && (
            <div className="grid grid-cols-1 gap-6 md:grid-cols-[1fr_auto]">
              <VideoInspectResult
                data={inspectData}
                isLoading={isInspecting}
                error={inspectError}
              />
              {!isInspecting && (
                <VideoCompress file={file} isInspecting={isInspecting} />
              )}
            </div>
          )}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
