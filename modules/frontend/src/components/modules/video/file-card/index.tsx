import type {
  InspectionData,
  ProcessingOptions,
} from "components/modules/video/types";
import { type FC, useState } from "react";

import {
  JobType,
  ProgressType,
  Status,
} from "components/modules/video/constants";
import { statusToAttachmentStateMap } from "components/modules/video/file-card/constants";
import { useObjectUrl } from "components/modules/video/hooks/useObjectUrl";
import { useVideoThumbnail } from "components/modules/video/hooks/useVideoThumbnail";
import { ProcessingOptionsSelect } from "components/modules/video/processing-options";
import { ProcessingSummary } from "components/modules/video/processing-summary";
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
import { Card } from "components/ui/card";
import { Progress } from "components/ui/progress";
import { Spinner } from "components/ui/spinner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "components/ui/tabs";
import { downloadFile } from "helpers/api/downloadFile";
import { DownloadIcon, VideoIcon } from "lucide-react";

enum FileCardTab {
  Preview = "preview",
  Details = "details",
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
  options: ProcessingOptions;
  appliedOptions?: ProcessingOptions;
  processingTimeMs?: number;
  onOptionsChange: (options: ProcessingOptions) => void;
}

export const FileCard: FC<FileCardProps> = ({
  file,
  progress,
  progressType,
  status,
  inspectData,
  jobType,
  error,
  processedData,
  options,
  appliedOptions,
  processingTimeMs,
  onOptionsChange,
}) => {
  const thumbnail = useVideoThumbnail(file);
  const previewUrl = useObjectUrl(processedData);
  const isPending = status === Status.Pending;
  const isInspecting = isPending && jobType === JobType.Inspect;
  const isProcessing = isPending && jobType === JobType.Compress;
  // Options apply to the next processing request, so hide them once it starts
  const showOptions = !processedData && !isProcessing;
  const hasPreview = !!previewUrl;
  const hasInspection = !!inspectData || isInspecting;
  const [selectedTab, setSelectedTab] = useState<FileCardTab>();
  const activeTab =
    selectedTab ?? (hasPreview ? FileCardTab.Preview : FileCardTab.Details);

  const handleDownloadFile = () => {
    if (processedData) {
      downloadFile(file, processedData);
    }
  };

  return (
    <Card className="flex flex-col w-full backdrop-blur-xs p-2">
      <Attachment
        state={statusToAttachmentStateMap[status]}
        className="flex-1 gap-4 w-full"
      >
        <AttachmentContent
          className={"flex gap-2 items-center justify-between p-2 flex-wrap"}
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
          <AttachmentTitle className={"text-base max-w-sm"}>
            {file.name}
          </AttachmentTitle>
          <div
            className={
              "flex flex-auto min-w-0 gap-4 items-center justify-between"
            }
          >
            {progressType && (
              <AttachmentDescription className={"text-base capitalize"}>
                {`${progressType} - ${progress}%`}
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
      {showOptions && (
        <ProcessingOptionsSelect options={options} onChange={onOptionsChange} />
      )}
      <Tabs
        value={activeTab}
        onValueChange={(value) => setSelectedTab(value as FileCardTab)}
      >
        <TabsList>
          <TabsTrigger value={FileCardTab.Preview} disabled={!previewUrl}>
            Preview
          </TabsTrigger>
          <TabsTrigger
            value={FileCardTab.Details}
            disabled={!hasInspection && !processedData}
          >
            Details
          </TabsTrigger>
        </TabsList>
        <TabsContent
          value={FileCardTab.Preview}
          className={"flex flex-col gap-1.5"}
        >
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
        <TabsContent value={FileCardTab.Details}>
          <VideoInspectResult data={inspectData} isLoading={isInspecting} />
        </TabsContent>
      </Tabs>
      {error && <VideoInspectError message={error} />}
    </Card>
  );
};
