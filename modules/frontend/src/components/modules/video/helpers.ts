import {
  errorsDict,
  FILE_SIZE_LIMIT,
  supportedMimeTypes,
} from "components/modules/video/constants";

export const validate_file = (
  file: File,
  showAlert: (message: string, severity: string) => void,
) => {
  if (!file.type) {
    showAlert(errorsDict.fileType, "error");
    return false;
  }

  if (!supportedMimeTypes.includes(file.type)) {
    showAlert(errorsDict.filesExtension, "error");

    return false;
  }

  if (file.size > FILE_SIZE_LIMIT) {
    showAlert(errorsDict.fileSize, "error");

    return false;
  }

  return true;
};
