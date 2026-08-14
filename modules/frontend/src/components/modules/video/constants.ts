export const FILE_SIZE_LIMIT: number = 100 * 1024 * 1024;
export const supportedMimeTypes: string[] = ["video/mp4"];
const supportedTypesLabel = supportedMimeTypes
  .map((el) => el.replace("image/", ""))
  .join(", ");
export const errorsDict: Record<
  "fileType" | "filesExtension" | "fileSize" | "fileName",
  string
> = {
  fileType: "File type in not supported",
  filesExtension: `Only the following formats are supported: ${supportedTypesLabel}`,
  fileSize: "Image size limit is 100 Mb",
  fileName: "File name is missing",
};
