"use client";

import type { ApiError } from "configs/types";
import { useMemo, useState } from "react";

import { getErrorsDict } from "components/modules/video/constants";
import {
  getMaxBodySize,
  validate_file,
} from "components/modules/video/helpers";
import { VideoAttachment } from "components/modules/video/video-attachment";
import { VideoCompress } from "components/modules/video/video-compress";
import { VideoDropZone } from "components/modules/video/video-drop-zone";
import { VideoInspectError } from "components/modules/video/video-inspect-error";
import { VideoInspectResult } from "components/modules/video/video-inspect-result";
import { inspectVideo } from "generated/client";
import { useAbortController } from "lib/hooks/useAbortController";
import { toast } from "sonner";

export default function VideoPage() {
  const [file, setFile] = useState<File | null>(null);

  const [inspectData, setInspectData] = useState<Record<
    string,
    unknown
  > | null>(null);
  const [isInspecting, setIsInspecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [uploadProgress, setUploadProgress] = useState(0);
  const abortController = useAbortController();
  const maxBodySize = useMemo(() => getMaxBodySize(), []);
  const errorsDict = useMemo(() => getErrorsDict(maxBodySize), [maxBodySize]);
  const showAlert = (msg: string) => {
    toast.error(msg);
  };

  async function handleFile(file: File) {
    if (!validate_file(file, errorsDict, showAlert)) {
      return;
    }

    setFile(file);
    setInspectData(null);
    setError(null);
    setIsInspecting(true);

    try {
      abortController.abort();
      abortController.init();

      const res = await inspectVideo({
        body: { video: file },
        onUploadProgress: (progressEvent) => {
          const total = progressEvent.total || progressEvent.bytes;
          const loaded = progressEvent.loaded;
          setUploadProgress(Math.round((loaded / total) * 100));
        },
        signal: abortController.ref.current?.signal,
      });

      if (res.error) {
        throw res;
      }

      setError(null);
      setInspectData(res.data as Record<string, unknown>);
    } catch (err) {
      const error = err as ApiError;
      const errorMessage =
        error.error || error.message || error.status || "Inspection failed.";

      toast.error(errorMessage);

      setError(errorMessage.toString());
    } finally {
      setIsInspecting(false);
      abortController.ref.current = null;
    }
  }

  function handleReset() {
    abortController.ref.current?.abort();

    setFile(null);
    setInspectData(null);
    setError(null);
  }

  return (
    <div className="flex flex-1 flex-col p-4 md:p-6 gap-6">
      <VideoDropZone
        file={file}
        onFile={handleFile}
        onReset={handleReset}
        disabled={isInspecting}
      />
      <div className={"flex gap-6 items-center"}>
        {!!file && (
          <VideoAttachment
            isInspecting={isInspecting}
            fileName={file.name}
            uploadProgress={uploadProgress}
            abort={abortController.abort}
          />
        )}
        {!!inspectData && (
          <VideoCompress
            file={file}
            isInspecting={isInspecting}
            abortController={abortController}
            setError={setError}
          />
        )}
      </div>
      {!isInspecting && error && <VideoInspectError message={error} />}
      {inspectData && (
        <VideoInspectResult data={inspectData} isLoading={isInspecting} />
      )}
    </div>
  );
}
