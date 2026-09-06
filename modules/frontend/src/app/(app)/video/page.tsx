"use client";

import { useCallback, useEffect, useState } from "react";

import { ProgressType } from "components/modules/video/constants";
import { ControlsBar } from "components/modules/video/controls-bar";
import { DropZone } from "components/modules/video/drop-zone";
import { FilesList } from "components/modules/video/files-list";
import { useWebSocket } from "components/modules/video/hooks/useWebSocket";
import { FileState, type FilesStateMap } from "components/modules/video/types";
import { getInspectVideoPromise } from "helpers/api/getInspectVideoPromise";
import { getCompressVideoPromise } from "helpers/api/getProcessingVideoPromise";
import { useAtomValue } from "jotai";
import { videoStore } from "store/video";

export default function VideoPage() {
  const videoState = useAtomValue(videoStore);
  const [files, setFiles] = useState<File[]>([]);
  const [filesStateMap, setFilesStateMap] = useState<FilesStateMap>({});

  const handleAddFiles = useCallback((newFiles: File[]) => {
    setFiles((files) => [...files, ...newFiles]);

    const newFilesState = newFiles.reduce<FilesStateMap>((acc, file) => {
      acc[file.name] = new FileState(file.name);
      return acc;
    }, {});

    setFilesStateMap((state) => ({
      ...state,
      ...newFilesState,
    }));
  }, []);

  const triggerUpdate = useCallback(() => {
    setFilesStateMap((state) => ({
      ...state,
    }));
  }, []);

  const handleReset = useCallback(() => {
    for (const file of files) {
      const fileState = filesStateMap[file.name];

      if (!fileState) {
        continue;
      }

      fileState.abortController.abort();
    }
    setFiles([]);
  }, [files, filesStateMap]);

  const handleCompressFiles = useCallback(async () => {
    const requests = [];

    for (const file of files) {
      const fileState = filesStateMap[file.name];

      if (!fileState) {
        continue;
      }

      requests.push(() =>
        getCompressVideoPromise(file, fileState, triggerUpdate),
      );
    }

    try {
      await Promise.all(requests.map((r) => r()));
    } catch (err) {
      console.log(err);
    }
  }, [files, filesStateMap, triggerUpdate]);

  const handleInspectFiles = useCallback(async () => {
    const requests = [];

    for (const file of files) {
      const fileState = filesStateMap[file.name];
      if (!fileState) {
        continue;
      }

      requests.push(() =>
        getInspectVideoPromise(file, fileState, triggerUpdate),
      );
    }

    try {
      await Promise.all(requests.map((r) => r()));
    } catch (err) {
      console.log(err);
    }
  }, [filesStateMap, triggerUpdate, files]);

  useEffect(() => {
    setFilesStateMap((state) => {
      Object.entries(videoState).forEach(([key, value]) => {
        state[key].progress = value.progress;
        state[key].progressType = value.done
          ? ProgressType.Processed
          : ProgressType.Processing;
      });

      return { ...state };
    });
  }, [videoState]);

  useWebSocket();

  return (
    <div className="flex flex-1 w-full justify-center">
      <div className={"flex flex-1 flex-col p-4 md:p-6 gap-8 max-w-200"}>
        {files.length ? (
          <>
            <ControlsBar
              compressFiles={handleCompressFiles}
              inspectFiles={handleInspectFiles}
              reset={handleReset}
              filesStateMap={filesStateMap}
            />
            <FilesList files={files} filesStateMap={filesStateMap} />
          </>
        ) : (
          <DropZone addFiles={handleAddFiles} />
        )}
      </div>
    </div>
  );
}
