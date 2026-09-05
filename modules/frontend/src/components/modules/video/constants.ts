import { formatBytes } from "lib/utils";

export const DEFAULT_MAX_BODY_SIZE: number = 10 * 1024 * 1024;
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
  Processing = "processing",
  Uploading = "uploading",
}

export const WS_RECONNECT_TIMEOUT_TIME = 4000;
export const WS_RECONNECT_ATTEMPTS = 4;
