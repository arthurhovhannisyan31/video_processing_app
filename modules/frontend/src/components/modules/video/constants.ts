import type { ProcessVideoPayload } from "generated/client";

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

// Values of the enums below must match `ProcessVideoPayload` in the generated client, which
// mirrors `process::configs` in the backend. The `*MatchesApi` checks at the end of this block fail
// to type-check if they drift.
export enum Codec {
  X264 = "libx264",
  Vp9 = "libvpx-vp9",
}

// Output container, mirrors backend `process::configs::extension`
export enum Format {
  Mp4 = "mp4",
  Mkv = "mkv",
  Webm = "webm",
  Mov = "mov",
}

export enum Preset {
  Fast = "superfast",
  Balanced = "medium",
  BestCompression = "slow",
}

// Quality level picked in the UI; the CRF value sent to the API depends on the codec
export enum Quality {
  High = "high",
  Standard = "standard",
  Low = "low",
}

type ApiCrf = ProcessVideoPayload["quality"];

const qualityCrfByCodec: Record<Codec, Record<Quality, ApiCrf>> = {
  [Codec.X264]: {
    [Quality.High]: "18",
    [Quality.Standard]: "23",
    [Quality.Low]: "28",
  },
  [Codec.Vp9]: {
    [Quality.High]: "24",
    [Quality.Standard]: "31",
    [Quality.Low]: "38",
  },
};

// webm only supports VP9; the other containers use x264
const codecByFormat: Record<Format, Codec> = {
  [Format.Mp4]: Codec.X264,
  [Format.Mkv]: Codec.X264,
  [Format.Mov]: Codec.X264,
  [Format.Webm]: Codec.Vp9,
};

export const getCodec = (format: Format): Codec => codecByFormat[format];

export const getQualityCrf = (codec: Codec, quality: Quality): ApiCrf =>
  qualityCrfByCodec[codec][quality];

// Only x264 has presets; VP9 ignores the value
export const hasPreset = (codec: Codec): boolean => codec === Codec.X264;

type IsExact<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;
export const codecMatchesApi: IsExact<
  `${Codec}`,
  ProcessVideoPayload["codec"]
> = true;
export const formatMatchesApi: IsExact<
  `${Format}`,
  ProcessVideoPayload["format"]
> = true;
export const presetMatchesApi: IsExact<
  `${Preset}`,
  ProcessVideoPayload["preset"]
> = true;

export const DEFAULT_FORMAT = Format.Mp4;
export const DEFAULT_PRESET = Preset.Balanced;
export const DEFAULT_QUALITY = Quality.Standard;

export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

export const formatOptions = [
  { value: Format.Mp4, label: "MP4" },
  { value: Format.Mkv, label: "MKV" },
  { value: Format.Mov, label: "MOV" },
  { value: Format.Webm, label: "WebM" },
] satisfies SelectOption<Format>[];

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
