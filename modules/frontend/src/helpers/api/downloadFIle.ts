export const downloadFIle = (file: File, blob: Blob) => {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  const stem = file.name.replace(/\.[^.]+$/, "");
  a.href = url;
  a.download = `${stem}_compressed.mp4`;
  a.click();
  URL.revokeObjectURL(url);
};
