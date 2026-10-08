import type { FilesStateMap } from "components/modules/video/types";
import { type FC, useMemo } from "react";

import { JobType, Status } from "components/modules/video/constants";
import { Button } from "components/ui/button";
import { Spinner } from "components/ui/spinner";
import { DownloadIcon } from "lucide-react";

export interface ControlsBarProps {
  filesStateMap: FilesStateMap;
  compressFiles: () => void;
  downloadAll: () => void;
  reset: () => void;
}

export const ControlsBar: FC<ControlsBarProps> = ({
  filesStateMap,
  compressFiles,
  downloadAll,
  reset,
}) => {
  const isCompressing = useMemo(
    () =>
      !!Object.values(filesStateMap).filter(
        (fileState) =>
          fileState.status === Status.Pending &&
          fileState.jobType === JobType.Compress,
      ).length,
    [filesStateMap],
  );

  const isInspecting = useMemo(
    () =>
      !!Object.values(filesStateMap).filter(
        (fileState) =>
          fileState.status === Status.Pending &&
          fileState.jobType === JobType.Inspect,
      ).length,
    [filesStateMap],
  );

  const hasProcessedFiles = useMemo(
    () =>
      Object.values(filesStateMap).some((fileState) => fileState.processedData),
    [filesStateMap],
  );

  return (
    <div className={"flex gap-4 w-full justify-between"}>
      <Button
        className="sm:w-auto h-10 text-base backdrop-blur-xs"
        onClick={compressFiles}
        disabled={isInspecting || isCompressing}
      >
        {isCompressing ? <Spinner /> : "Compress files"}
      </Button>
      <Button
        className="sm:w-auto h-10 text-base backdrop-blur-xs"
        onClick={downloadAll}
        disabled={isCompressing || !hasProcessedFiles}
      >
        <DownloadIcon />
        Download all
      </Button>
      <Button
        variant={"destructive"}
        className="sm:w-auto h-10 text-base backdrop-blur-xs"
        onClick={reset}
      >
        Reset
      </Button>
    </div>
  );
};
