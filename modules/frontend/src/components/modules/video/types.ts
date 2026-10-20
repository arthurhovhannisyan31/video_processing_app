import {
  DEFAULT_FORMAT,
  DEFAULT_PRESET,
  DEFAULT_QUALITY,
  type Format,
  type JobType,
  type Preset,
  type ProgressType,
  type Quality,
  Status,
} from "components/modules/video/constants";

export type InspectionData = Record<string, unknown>;

export interface ProcessingOptions {
  format: Format;
  // Sent for every codec, but only used by x264
  preset: Preset;
  quality: Quality;
}

export class FileState {
  public options: ProcessingOptions = {
    format: DEFAULT_FORMAT,
    preset: DEFAULT_PRESET,
    quality: DEFAULT_QUALITY,
  };

  constructor(
    public name: string,
    public progress: number = 0,
    public abortController: AbortController = new AbortController(),
    public status: Status = Status.Stale,
    public error: string = "",
    public jobType?: JobType,
    public progressType?: ProgressType,
    public inspectionData?: InspectionData,
    public processedData?: Blob,
    // Snapshot of `options` that produced `processedData`
    public appliedOptions?: ProcessingOptions,
    // Wall-clock time of the processing request (upload + processing + download)
    public processingTimeMs?: number,
  ) {}
}

export type FilesStateMap = Record<string, FileState>;
