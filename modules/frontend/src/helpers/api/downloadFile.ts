import type { Format } from "components/modules/video/constants";

export const downloadFile = (file: File, blob: Blob, format: Format) => {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  const stem = file.name.replace(/\.[^.]+$/, "");
  a.href = url;
  a.download = `${stem}_compressed.${format}`;
  a.click();
  requestIdleCallback(() => {
    URL.revokeObjectURL(url);
  });
};
