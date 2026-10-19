export const DEFAULT_MAX_BODY_SIZE: number = 10 * 1024 * 1024;
export const WS_RECONNECT_TIMEOUT_TIME = 4000;
export const WS_RECONNECT_ATTEMPTS = 4;
export const MAX_FILES_COUNT = 3;
export const DOWNLOAD_ALL_DELAY_MS = 200;

export const supportedMimeTypes: string[] = ["video/mp4"];
const supportedTypesLabel = supportedMimeTypes
  .map((el) => el.replace("video/", ""))
  .join(", ");

export type ErrorsDict = Record<
  "fileType" | "filesExtension" | "fileSize" | "fileName",
  string
>;
export const getErrorsDict = (maxBodySizeMB: string): ErrorsDict => ({
  fileType: "File type is not supported",
  filesExtension: `Only the following formats are supported: ${supportedTypesLabel}`,
  fileSize: `File size limit is ${maxBodySizeMB}`,
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

export enum Codec {
  X264 = "libx264",
}

// Values must match `ProcessVideoPayload` in the generated client; the processing
// request body fails to type-check if they drift.
export enum Preset {
  Fast = "superfast",
  Balanced = "medium",
  BestCompression = "slow",
}

export enum Quality {
  High = "18",
  Standard = "23",
  Low = "28",
}

export const DEFAULT_PRESET = Preset.Balanced;
export const DEFAULT_QUALITY = Quality.Standard;

export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

export const presetOptions = [
  { value: Preset.Fast, label: "Fast" },
  { value: Preset.Balanced, label: "Balanced" },
  { value: Preset.BestCompression, label: "Best compression" },
] satisfies SelectOption<Preset>[];

export const qualityOptions = [
  { value: Quality.High, label: "High" },
  { value: Quality.Standard, label: "Standard" },
  { value: Quality.Low, label: "Low" },
] satisfies SelectOption<Quality>[];

export const getOptionLabel = <T extends string>(
  options: SelectOption<T>[],
  value: T,
): string => options.find((option) => option.value === value)?.label ?? value;
