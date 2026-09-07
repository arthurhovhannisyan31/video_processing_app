import {
  type JobType,
  type ProgressType,
  Status,
} from "components/modules/video/constants";

export type InspectionData = Record<string, unknown>;

export class FileState {
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
  ) {}
}

export type FilesStateMap = Record<string, FileState>;
