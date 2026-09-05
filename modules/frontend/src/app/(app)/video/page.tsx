"use client";

import type { InspectionData } from "components/modules/video/types";
import { useCallback, useEffect, useState } from "react";

import { JobType } from "components/modules/video/constants";
import useHandleFileUpload from "components/modules/video/hooks/useHandleFileUpload";
import { useWebSocket } from "components/modules/video/hooks/useWebSocket";
import { VideoAttachment } from "components/modules/video/video-attachment";
import { VideoCompress } from "components/modules/video/video-compress";
import { VideoDropZone } from "components/modules/video/video-drop-zone";
import { VideoInspectError } from "components/modules/video/video-inspect-error";
import { VideoInspectResult } from "components/modules/video/video-inspect-result";
import { Progress } from "components/ui/progress";
import { useAbortController } from "hooks/useAbortController";
import { useAtomValue } from "jotai";
import { videoStore } from "store/video";

export default function VideoPage() {
  const [file, setFile] = useState<File | null>(null);
  const [inspectData, setInspectData] = useState<InspectionData | null>(null);
  const [jobType, setJobType] = useState<JobType>();
  const [isInspecting, setInspecting] = useState(false);
  const [isProcessing, setProcessing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const videoState = useAtomValue(videoStore);

  const abortController = useAbortController();

  const handleFile = useHandleFileUpload(
    setJobType,
    setFile,
    setInspectData,
    setError,
    setInspecting,
    abortController,
    setProgress,
  );

  function handleReset() {
    abortController.ref.current?.abort();

    setFile(null);
    setInspectData(null);
    setError(null);
  }

  const handleProcessingStart = useCallback(() => {
    abortController.abort();
    abortController.init();
    setJobType(JobType.Processing);
    setProcessing(true);
    setProgress(0);
  }, [abortController.abort, abortController.init]);

  const handleProcessingSettled = useCallback(() => {
    abortController.ref.current = null;
    setProcessing(false);
  }, [abortController.ref]);

  useEffect(() => {
    setProgress(videoState.progress);
    if (videoState.done) {
      setProcessing(false);
    }
  }, [videoState]);
  useWebSocket();

  return (
    <div className="flex flex-1 flex-col p-4 md:p-6 gap-6">
      <VideoDropZone
        file={file}
        onFile={handleFile}
        onReset={handleReset}
        disabled={isInspecting}
      />
      <div className={"flex flex-col gap-6 items-center"}>
        <div className={"flex gap-6 items-center w-[75%]"}>
          {!!file && (
            <VideoAttachment
              isLoading={isInspecting}
              fileName={file.name}
              progress={progress}
              abort={abortController.abort}
              jobType={jobType}
            />
          )}
          {!!inspectData && (
            <VideoCompress
              file={file}
              isInspecting={isProcessing}
              abortController={abortController}
              setError={setError}
              onSettled={handleProcessingSettled}
              onStart={handleProcessingStart}
            />
          )}
        </div>
        {isProcessing && <Progress className="w-[75%]" value={progress} />}
      </div>
      {!isInspecting && error && <VideoInspectError message={error} />}
      {inspectData && (
        <VideoInspectResult data={inspectData} isLoading={isInspecting} />
      )}
    </div>
  );
}
