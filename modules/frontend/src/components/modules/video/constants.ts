import { formatBytes } from "lib/utils";

export const DEFAULT_MAX_BODY_SIZE: number = 10 * 1024 * 1024;
export const WS_RECONNECT_TIMEOUT_TIME = 4000;
export const WS_RECONNECT_ATTEMPTS = 4;
export const MAX_FILES_COUNT = 4;

export const supportedMimeTypes: string[] = ["video/mp4"];
const supportedTypesLabel = supportedMimeTypes
  .map((el) => el.replace("image/", ""))
  .join(", ");

export type ErrorsDict = Record<
  "fileType" | "filesExtension" | "fileSize" | "fileName",
  string
>;
export const getErrorsDict = (maxBodySizeMB: number): ErrorsDict => ({
  fileType: "File type in not supported",
  filesExtension: `Only the following formats are supported: ${supportedTypesLabel}`,
  fileSize: `Image size limit is ${formatBytes(maxBodySizeMB)}`,
  fileName: "File name is missing",
});

export enum JobType {
  Inspect = "inspect",
  Compress = "compress",
}

export enum Status {
  Stale = "stale",
  Pending = "pending",
  Done = "done",
  Error = "error",
}

export enum ProgressType {
  Uploading = "uploading",
  Uploaded = "uploaded",
  Processing = "processing",
  Processed = "processed",
}

export enum Operation {
  Compress = "compress",
}
