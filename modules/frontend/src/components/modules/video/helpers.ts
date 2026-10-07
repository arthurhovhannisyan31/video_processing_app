import {
  DEFAULT_MAX_BODY_SIZE,
  type ErrorsDict,
  supportedMimeTypes,
} from "components/modules/video/constants";
import { MAX_BODY_SIZE } from "configs/constants";

export const getMaxBodySizeBytes = () => {
  const bodySize = +(MAX_BODY_SIZE ?? 0);

  if (Number.isFinite(bodySize) && bodySize > 0) {
    return bodySize;
  }

  return DEFAULT_MAX_BODY_SIZE;
};

export const validateFile = (
  file: File,
  errorsDict: ErrorsDict,
  showAlert: (message: string) => void,
) => {
  if (!file.type) {
    showAlert(errorsDict.fileType);
    return false;
  }

  if (!supportedMimeTypes.includes(file.type)) {
    showAlert(errorsDict.filesExtension);

    return false;
  }

  if (file.size > getMaxBodySizeBytes()) {
    showAlert(errorsDict.fileSize);

    return false;
  }

  return true;
};
