import type { FileState } from "components/modules/video/types";
import type { ApiError } from "configs/types";

import {
  JobType,
  Operation,
  ProgressType,
  Status,
} from "components/modules/video/constants";
import { processVideo } from "generated/client";
import { toast } from "sonner";

export const getCompressVideoPromise = (
  file: File,
  fileState: FileState,
  triggerUpdate: () => void,
) =>
  // Use Promise wrapper to use Promise.all
  new Promise((res, rej) => {
    (async () => {
      try {
        fileState.jobType = JobType.Compress;
        fileState.status = Status.Pending;
        fileState.progressType = ProgressType.Uploaded;
        triggerUpdate();

        const response = await processVideo({
          body: { video: file, operation: Operation.Compress },
          responseType: "blob",
          onUploadProgress: (progressEvent) => {
            const total = progressEvent.total || progressEvent.bytes;
            const loaded = progressEvent.loaded;
            fileState.progress = Math.round((loaded / total) * 100);

            triggerUpdate();
          },
          signal: fileState.abortController.signal,
        });

        if (response.error) {
          throw response;
        }

        fileState.progressType = ProgressType.Uploaded;
        fileState.processedData = response.data as unknown as Blob;
        fileState.status = Status.Done;

        res(null);
      } catch (err) {
        fileState.status = Status.Error;

        const error = err as ApiError;
        const errorMessage = (
          error.message ||
          error.status ||
          "Inspection failed."
        ).toString();
        fileState.error = errorMessage;
        toast.error(errorMessage);

        rej(err);
      } finally {
        fileState.jobType = undefined;
        triggerUpdate();
      }
    })();
  });
