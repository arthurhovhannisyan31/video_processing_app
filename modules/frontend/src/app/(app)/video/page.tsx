"use client";

import { useCallback, useEffect, useState } from "react";

import {
  DOWNLOAD_ALL_DELAY_MS,
  ProgressType,
} from "components/modules/video/constants";
import { ControlsBar } from "components/modules/video/controls-bar";
import { DropZone } from "components/modules/video/drop-zone";
import { FilesList } from "components/modules/video/files-list";
import { useWebSocket } from "components/modules/video/hooks/useWebSocket";
import { FileState, type FilesStateMap } from "components/modules/video/types";
import { downloadFile } from "helpers/api/downloadFile";
import { getInspectVideoPromise } from "helpers/api/getInspectVideoPromise";
import { getCompressVideoPromise } from "helpers/api/getProcessingVideoPromise";
import { useAtomValue } from "jotai";
import { store } from "store";
import { videoStore } from "store/video";

export default function VideoPage() {
  const videoState = useAtomValue(videoStore);
  const [files, setFiles] = useState<File[]>([]);
  const [filesStateMap, setFilesStateMap] = useState<FilesStateMap>({});
  const { wsReconnect } = useWebSocket();

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
    setFilesStateMap({});
    store.set(videoStore, {});
  }, [files, filesStateMap]);

  const handleCompressFiles = useCallback(async () => {
    await wsReconnect();

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
      await Promise.allSettled(requests.map((r) => r()));
    } catch (err) {
      console.error(err);
    }
  }, [files, filesStateMap, triggerUpdate, wsReconnect]);

  const handleDownloadAll = useCallback(async () => {
    for (const file of files) {
      const processedData = filesStateMap[file.name]?.processedData;

      if (!processedData) {
        continue;
      }

      downloadFile(file, processedData);
      // Browsers drop back-to-back programmatic downloads without a short gap.
      await new Promise((resolve) =>
        setTimeout(resolve, DOWNLOAD_ALL_DELAY_MS),
      );
    }
  }, [files, filesStateMap]);

  const handleInspectFiles = async () => {
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
      console.error(err);
    }
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: Only trigger to files change
  useEffect(() => {
    if (files.length) {
      handleInspectFiles();
    }
  }, [files]);

  useEffect(() => {
    setFilesStateMap((state) => {
      Object.entries(videoState).forEach(([key, value]) => {
        if (!state[key]) {
          return;
        }
        state[key].progress = value.progress;
        state[key].progressType = value.done
          ? ProgressType.Processed
          : ProgressType.Processing;
      });

      return { ...state };
    });
  }, [videoState]);

  return (
    <div className="flex flex-1 w-full justify-center">
      <div className={"flex flex-1 flex-col p-4 md:p-6 gap-8 max-w-200"}>
        {files.length ? (
          <>
            <ControlsBar
              compressFiles={handleCompressFiles}
              downloadAll={handleDownloadAll}
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
