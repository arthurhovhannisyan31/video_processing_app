import type { AbortControllerResult } from "hooks/useAbortController";
import { type FC, useMemo } from "react";

import { JobType } from "components/modules/video/constants";
import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentMedia,
  type AttachmentStateType,
  AttachmentTitle,
} from "components/ui/attachment";
import { Spinner } from "components/ui/spinner";
import { VideoIcon, XIcon } from "lucide-react";

export interface VideoAttachmentProps {
  isLoading?: boolean;
  isError?: boolean;
  fileName: string;
  progress?: number;
  abort: AbortControllerResult["abort"];
  jobType?: JobType;
}

export const VideoAttachment: FC<VideoAttachmentProps> = ({
  fileName,
  isLoading,
  isError,
  progress,
  abort,
  jobType,
}) => {
  const attachmentState = useMemo<AttachmentStateType>(() => {
    if (isLoading) {
      return "processing";
    }
    if (isError) {
      return "error";
    }
    return "done";
  }, [isError, isLoading]);

  const description = useMemo(() => {
    switch (jobType) {
      case JobType.Processing: {
        return `${isLoading ? "Processing" : "Processed"}· ${progress}%`;
      }
      case JobType.Uploading: {
        return `${isLoading ? "Uploading" : "Uploaded"}· ${progress}%`;
      }
      default:
        return "";
    }
  }, [jobType, isLoading, progress]);

  return (
    <Attachment state={attachmentState} className="flex-1 gap-4">
      <AttachmentMedia>
        {isLoading ? <Spinner /> : <VideoIcon />}
      </AttachmentMedia>
      <AttachmentContent className={"flex gap-2 items-center"}>
        <AttachmentTitle className={"text-base max-w-sm"}>
          {fileName}
        </AttachmentTitle>
        <AttachmentDescription className={"text-base"}>
          {description}
        </AttachmentDescription>
      </AttachmentContent>
      {isLoading && (
        <AttachmentActions>
          <AttachmentAction
            aria-label="Cancel upload"
            className={"w-8 h-8"}
            onClick={abort}
          >
            <XIcon className={"size-4"} />
          </AttachmentAction>
        </AttachmentActions>
      )}
    </Attachment>
  );
};
