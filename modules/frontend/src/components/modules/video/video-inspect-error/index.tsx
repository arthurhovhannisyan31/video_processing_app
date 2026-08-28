import type { FC } from "react";

export interface VideoInspectErrorProps {
  message?: string;
}

export const VideoInspectError: FC<VideoInspectErrorProps> = ({ message }) => {
  return (
    <div className="rounded-lg border border-destructive/30 bg-destructive/10 p-4">
      <p className="text-destructive text-sm">{message}</p>
    </div>
  );
};
