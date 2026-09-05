import type { InspectionData } from "components/modules/video/types";
import type { ApiError } from "configs/types";
import type { AbortControllerResult } from "hooks/useAbortController";
import { useCallback, useMemo } from "react";

import { getErrorsDict, JobType } from "components/modules/video/constants";
import {
  getMaxBodySize,
  validate_file,
} from "components/modules/video/helpers";
import { inspectVideo } from "generated/client";
import { toast } from "sonner";

const useHandleFileUpload = (
  setJobType: (val: JobType) => void,
  setFile: (file: File) => void,
  setInspectData: (val: InspectionData | null) => void,
  setError: (val: string | null) => void,
  setLoading: (val: boolean) => void,
  abortController: AbortControllerResult,
  setProgress: (val: number) => void,
) => {
  const showAlert = useCallback((msg: string) => {
    toast.error(msg);
  }, []);
  const maxBodySize = useMemo(() => getMaxBodySize(), []);
  const errorsDict = useMemo(() => getErrorsDict(maxBodySize), [maxBodySize]);

  return useCallback(
    async (file: File) => {
      if (!validate_file(file, errorsDict, showAlert)) {
        return;
      }

      setJobType(JobType.Uploading);
      setFile(file);
      setInspectData(null);
      setError(null);
      setLoading(true);

      try {
        abortController.abort();
        abortController.init();

        const res = await inspectVideo({
          body: { video: file },
          onUploadProgress: (progressEvent) => {
            const total = progressEvent.total || progressEvent.bytes;
            const loaded = progressEvent.loaded;
            setProgress(Math.round((loaded / total) * 100));
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
        const errorMessage = (
          error.message ||
          error.status ||
          error.error ||
          "Inspection failed."
        ).toString();

        toast.error(errorMessage);

        setError(errorMessage);
      } finally {
        setLoading(false);
        abortController.ref.current = null;
      }
    },
    [
      abortController,
      errorsDict,
      setError,
      setFile,
      setInspectData,
      setJobType,
      setLoading,
      showAlert,
      setProgress,
    ],
  );
};
export default useHandleFileUpload;
