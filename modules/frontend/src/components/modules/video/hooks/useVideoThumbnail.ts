import { useEffect, useState } from "react";

const THUMBNAIL_WIDTH = 160;
const THUMBNAIL_QUALITY = 0.7;
const MAX_SEEK_SECONDS = 1;

export const useVideoThumbnail = (file: File): string | undefined => {
  const [thumbnail, setThumbnail] = useState<string>();

  useEffect(() => {
    let cancelled = false;
    const objectUrl = URL.createObjectURL(file);
    const video = document.createElement("video");
    video.muted = true;
    video.playsInline = true;
    video.preload = "metadata";

    const handleLoadedMetadata = () => {
      video.currentTime = Math.min(MAX_SEEK_SECONDS, video.duration * 0.1);
    };

    const handleSeeked = () => {
      if (cancelled || !video.videoWidth || !video.videoHeight) {
        return;
      }

      const canvas = document.createElement("canvas");
      canvas.width = THUMBNAIL_WIDTH;
      canvas.height = Math.round(
        (video.videoHeight / video.videoWidth) * THUMBNAIL_WIDTH,
      );
      canvas
        .getContext("2d")
        ?.drawImage(video, 0, 0, canvas.width, canvas.height);

      setThumbnail(canvas.toDataURL("image/jpeg", THUMBNAIL_QUALITY));
    };

    video.addEventListener("loadedmetadata", handleLoadedMetadata);
    video.addEventListener("seeked", handleSeeked);
    video.src = objectUrl;

    return () => {
      cancelled = true;
      video.removeEventListener("loadedmetadata", handleLoadedMetadata);
      video.removeEventListener("seeked", handleSeeked);
      video.removeAttribute("src");
      video.load();
      URL.revokeObjectURL(objectUrl);
      setThumbnail(undefined);
    };
  }, [file]);

  return thumbnail;
};
