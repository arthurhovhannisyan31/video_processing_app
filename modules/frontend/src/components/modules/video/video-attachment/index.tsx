import type { AbortControllerResult } from "lib/hooks/useAbortController";
import type { FC } from "react";

import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentMedia,
  AttachmentTitle,
} from "components/ui/attachment";
import { Spinner } from "components/ui/spinner";
import { VideoIcon, XIcon } from "lucide-react";

export interface VideoAttachmentProps {
  isInspecting?: boolean;
  fileName: string;
  uploadProgress?: number;
  abort: AbortControllerResult["abort"];
}

export const VideoAttachment: FC<VideoAttachmentProps> = ({
  fileName,
  isInspecting,
  uploadProgress,
  abort,
}) => {
  return (
    // TODO Change state based on current stage
    <Attachment state="uploading" className="w-fit gap-4">
      <AttachmentMedia>
        {isInspecting ? <Spinner /> : <VideoIcon />}
      </AttachmentMedia>
      <AttachmentContent className={"flex gap-2 items-center"}>
        <AttachmentTitle className={"text-base max-w-sm"}>
          {fileName}
        </AttachmentTitle>
        <AttachmentDescription className={"text-base"}>
          {isInspecting ? "Uploading" : "Uploaded"}· {uploadProgress}%
        </AttachmentDescription>
      </AttachmentContent>
      {isInspecting && (
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
