"use client";

import type { AxiosError } from "axios";
import { useState } from "react";

import { VideoCompress } from "components/modules/video/video-compress";
import { VideoDropZone } from "components/modules/video/video-drop-zone";
import { VideoInspectResult } from "components/modules/video/video-inspect-result";
import { inspectVideo } from "generated/client";
import { toast } from "sonner";

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
      const res = await inspectVideo({ body: { video: f } });

      if (res.error) {
        throw res.error;
      }

      setInspectData(res.data as Record<string, unknown>);
    } catch (err) {
      const error = err as AxiosError;
      const errorMessage = error.message || error.status;

      toast.error(errorMessage);

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
          {!isInspecting && !inspectError && (
            <VideoCompress file={file} isInspecting={isInspecting} />
          )}
        </div>
      )}
    </div>
  );
}
