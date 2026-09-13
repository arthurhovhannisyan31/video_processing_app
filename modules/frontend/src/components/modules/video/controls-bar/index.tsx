import type { FilesStateMap } from "components/modules/video/types";
import { type FC, useMemo } from "react";

import { JobType, Status } from "components/modules/video/constants";
import { Button } from "components/ui/button";
import { Spinner } from "components/ui/spinner";

export interface ControlsBarProps {
  filesStateMap: FilesStateMap;
  compressFiles: () => void;
  reset: () => void;
}

export const ControlsBar: FC<ControlsBarProps> = ({
  filesStateMap,
  compressFiles,
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

  return (
    <div className={"flex gap-4 w-full justify-center"}>
      <Button
        className="sm:w-auto h-10 text-base backdrop-blur-xs"
        onClick={compressFiles}
        disabled={isInspecting || isCompressing}
      >
        {isCompressing ? <Spinner /> : "Compress files"}
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
