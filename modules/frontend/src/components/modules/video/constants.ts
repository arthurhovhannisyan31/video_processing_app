export const DEFAULT_MAX_BODY_SIZE: number = 20 * 1024 * 1024;
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
  fileSize: `Image size limit is ${maxBodySizeMB} Mb`,
  fileName: "File name is missing",
});
