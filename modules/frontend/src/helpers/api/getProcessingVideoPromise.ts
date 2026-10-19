import type { FileState } from "components/modules/video/types";
import type { ApiError } from "configs/types";

import {
  Codec,
  JobType,
  Operation,
  ProgressType,
  Status,
} from "components/modules/video/constants";
import { processVideo } from "generated/client";
import { getErrorMessage } from "helpers/utils";
import { toast } from "sonner";

export const getCompressVideoPromise = async (
  file: File,
  fileState: FileState,
  triggerUpdate: () => void,
) => {
  fileState.jobType = JobType.Compress;
  fileState.status = Status.Pending;
  fileState.progressType = ProgressType.Uploaded;
  triggerUpdate();

  // Capture options at request time so later edits don't change what's reported
  const requestOptions = { ...fileState.options };
  const startedAt = performance.now();

  try {
    const response = await processVideo({
      // Fields are sent in key order; the API requires `codec` before `preset`
      body: {
        video: file,
        operation: Operation.Compress,
        codec: Codec.X264,
        preset: requestOptions.preset,
        quality: requestOptions.quality,
      },
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
    fileState.appliedOptions = { ...requestOptions };
    fileState.processingTimeMs = performance.now() - startedAt;
    fileState.status = Status.Done;
  } catch (err) {
    fileState.status = Status.Error;

    const error = err as ApiError;
    const errorMessage = getErrorMessage(error, "Processing failed.");
    fileState.error = errorMessage;
    toast.error(errorMessage);

    throw err;
  } finally {
    fileState.jobType = undefined;
    triggerUpdate();
  }
};
