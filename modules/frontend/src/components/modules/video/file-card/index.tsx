import type { InspectionData } from "components/modules/video/types";
import { type FC, useState } from "react";

import {
  JobType,
  ProgressType,
  Status,
} from "components/modules/video/constants";
import { statusToAttachmentStateMap } from "components/modules/video/file-card/constants";
import { useObjectUrl } from "components/modules/video/hooks/useObjectUrl";
import { useVideoThumbnail } from "components/modules/video/hooks/useVideoThumbnail";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { downloadFile } from "helpers/api/downloadFile";
import { formatBytesToMB } from "lib/utils";
import { ArrowLeft, DownloadIcon, VideoIcon } from "lucide-react";

enum FileCardTab {
  Preview = "preview",
  Inspection = "inspection",
}

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
  const thumbnail = useVideoThumbnail(file);
  const previewUrl = useObjectUrl(processedData);
  const isPending = status === Status.Pending;
  const isInspecting = isPending && jobType === JobType.Inspect;
  const hasPreview = !!previewUrl;
  const hasInspection = !!inspectData || isInspecting;

  const [selectedTab, setSelectedTab] = useState<FileCardTab>();
  // Follow the newest available result until the user picks a tab.
  const activeTab =
    selectedTab ?? (hasPreview ? FileCardTab.Preview : FileCardTab.Inspection);

  const handleDownloadFile = () => {
    if (processedData) {
      downloadFile(file, processedData);
    }
  };

  return (
    <div className={"flex flex-col w-full gap-1.5 backdrop-blur-xs"}>
      <Attachment
        state={statusToAttachmentStateMap[status]}
        className="flex-1 gap-4 w-full"
      >
        <AttachmentMedia variant={!isPending && thumbnail ? "image" : "icon"}>
          {isPending ? (
            <Spinner />
          ) : thumbnail ? (
            <img src={thumbnail} alt={file.name} />
          ) : (
            <VideoIcon />
          )}
        </AttachmentMedia>
        <AttachmentContent
          className={"flex gap-2 items-center justify-between p-2"}
        >
          <AttachmentTitle className={"text-base max-w-sm"}>
            {file.name}
          </AttachmentTitle>
          <div className={"flex gap-4 items-center"}>
            {progressType && (
              <AttachmentDescription className={"text-base capitalize"}>
                {`${progressType} - ${progress}%`}
              </AttachmentDescription>
            )}
            <AttachmentDescription className={"text-base"}>
              {formatBytesToMB(file.size, 2)}
            </AttachmentDescription>
            {processedData && (
              <AttachmentDescription className={"text-base flex gap-4"}>
                <ArrowLeft className={"rotate-180 w-4"} />
                {formatBytesToMB(processedData.size, 2)}
              </AttachmentDescription>
            )}
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
          </div>
        </AttachmentContent>
        {progressType === ProgressType.Processing && (
          <Progress className="w-full" value={progress} />
        )}
      </Attachment>
      {(hasPreview || hasInspection) && (
        <Tabs
          value={activeTab}
          onValueChange={(value) => setSelectedTab(value as FileCardTab)}
        >
          <TabsList>
            <TabsTrigger value={FileCardTab.Preview} disabled={!hasPreview}>
              Preview
            </TabsTrigger>
            <TabsTrigger
              value={FileCardTab.Inspection}
              disabled={!hasInspection}
            >
              Inspection
            </TabsTrigger>
          </TabsList>
          <TabsContent value={FileCardTab.Preview}>
            {previewUrl && (
              <video
                src={previewUrl}
                poster={thumbnail}
                controls
                playsInline
                preload={"metadata"}
                className={"w-full max-h-96 rounded-lg border bg-black"}
              >
                <track kind="captions" />
              </video>
            )}
          </TabsContent>
          <TabsContent value={FileCardTab.Inspection}>
            <VideoInspectResult data={inspectData} isLoading={isInspecting} />
          </TabsContent>
        </Tabs>
      )}
      {error && <VideoInspectError message={error} />}
    </div>
  );
};
export default FileCard;
