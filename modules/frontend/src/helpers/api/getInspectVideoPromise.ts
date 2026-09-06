import type { FileState } from "components/modules/video/types";
import type { ApiError } from "configs/types";

import {
  JobType,
  ProgressType,
  Status,
} from "components/modules/video/constants";
import { inspectVideo } from "generated/client";
import { toast } from "sonner";

export const getInspectVideoPromise = (
  file: File,
  fileState: FileState,
  triggerUpdate: () => void,
) =>
  // Use Promise wrapper to use Promise.all
  new Promise((res, rej) => {
    (async () => {
      try {
        fileState.jobType = JobType.Inspect;
        fileState.status = Status.Pending;
        fileState.progressType = ProgressType.Uploading;
        triggerUpdate();

        const response = await inspectVideo({
          body: { video: file },
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
        fileState.inspectionData = response.data;
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
