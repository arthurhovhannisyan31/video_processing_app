import type { InspectionData } from "components/modules/video/types";
import type { FC } from "react";

import {
  JobType,
  ProgressType,
  Status,
} from "components/modules/video/constants";
import { statusToAttachmentStateMap } from "components/modules/video/file-card/constants";
import { VideoInspectError } from "components/modules/video/video-inspect-error";
import { VideoInspectResult } from "components/modules/video/video-inspect-result";
import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentMedia,
  AttachmentTitle,
} from "components/ui/attachment";
import { Button } from "components/ui/button";
import { Progress } from "components/ui/progress";
import { Spinner } from "components/ui/spinner";
import { downloadFIle } from "helpers/api/downloadFIle";
import { formatBytes } from "lib/utils";
import { DownloadIcon, VideoIcon } from "lucide-react";

export interface FileCardProps {
  file: File;
  progress: number;
  progressType?: ProgressType;
  status: Status;
  inspectData?: InspectionData;
  jobType?: JobType;
  error?: string;
  processedData?: Blob;
}

const FileCard: FC<FileCardProps> = ({
  file,
  progress,
  progressType,
  status,
  inspectData,
  jobType,
  error,
  processedData,
}) => {
  const handleDownloadFile = () => {
    if (processedData) {
      downloadFIle(file, processedData);
    }
  };

  return (
    <div className={"flex flex-col w-full gap-1.5"}>
      <Attachment
        state={statusToAttachmentStateMap[status]}
        className="flex-1 gap-4 w-full"
      >
        <AttachmentMedia>
          {status === Status.Pending ? <Spinner /> : <VideoIcon />}
        </AttachmentMedia>
        <AttachmentContent
          className={"flex gap-2 items-center justify-between p-2"}
        >
          <AttachmentTitle className={"text-base max-w-sm"}>
            {file.name}
          </AttachmentTitle>
          {progressType && (
            <AttachmentDescription className={"text-base capitalize"}>
              {`${progressType} - ${progress}%`}
            </AttachmentDescription>
          )}
          <AttachmentDescription className={"text-base"}>
            {formatBytes(file.size)}
          </AttachmentDescription>
          {processedData && (
            <AttachmentActions>
              <AttachmentAction className={"outline"}>
                <Button
                  className={"outline"}
                  size={"icon-lg"}
                  onClick={handleDownloadFile}
                >
                  <DownloadIcon />
                </Button>
              </AttachmentAction>
            </AttachmentActions>
          )}
        </AttachmentContent>
        {progressType === ProgressType.Processing && (
          <Progress className="w-full" value={progress} />
        )}
      </Attachment>
      <VideoInspectResult
        data={inspectData}
        isLoading={status === Status.Pending && jobType === JobType.Inspect}
      />
      {error && <VideoInspectError message={error} />}
    </div>
  );
};
export default FileCard;
