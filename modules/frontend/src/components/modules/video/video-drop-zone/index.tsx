"use client";

import { useMemo, useRef, useState } from "react";

import { Cancel01Icon, Upload01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { getErrorsDict } from "components/modules/video/constants";
import {
  getMaxBodySize,
  validate_file,
} from "components/modules/video/helpers";
import { toast } from "sonner";

interface VideoDropZoneProps {
  file: File | null;
  onFile: (file: File) => void;
  onReset: () => void;
  disabled?: boolean;
}

export function VideoDropZone({
  file,
  onFile,
  onReset,
  disabled,
}: VideoDropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const showAlert = (msg: string) => {
    toast.error(msg);
  };
  const maxBodySize = useMemo(() => getMaxBodySize(), []);
  const errorsDict = useMemo(() => getErrorsDict(maxBodySize), [maxBodySize]);

  function handleDrop(e: React.DragEvent<HTMLDivElement>) {
    e.preventDefault();
    setIsDragging(false);
    if (disabled) return;

    const file = e.dataTransfer.files[0];
    if (!file) return;

    if (!validate_file(file, errorsDict, showAlert)) {
      return;
    }

    setError(null);
    onFile(file);
  }

  function handleDragOver(e: React.DragEvent<HTMLDivElement>) {
    e.preventDefault();
    if (!disabled) setIsDragging(true);
  }

  function handleDragLeave() {
    setIsDragging(false);
  }

  function handleClick() {
    if (!disabled && !file) inputRef.current?.click();
  }

  function handleInputChange(e: React.ChangeEvent<HTMLInputElement>) {
    const selected = e.target.files?.[0];
    if (!selected) return;
    setError(null);
    onFile(selected);
    e.target.value = "";
  }

  function formatBytes(bytes: number) {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  return (
    <div className="flex flex-col gap-2">
      {/* biome-ignore lint/a11y/useKeyWithClickEvents: not relevant */}
      {/** biome-ignore lint/a11y/noStaticElementInteractions: not relevant */}
      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onClick={handleClick}
        className={[
          "flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed p-10 transition-colors",
          isDragging ? "border-primary bg-primary/5" : "border-border",
          !file && !disabled
            ? "cursor-pointer hover:border-primary/60 hover:bg-muted/40"
            : "",
          disabled ? "opacity-50" : "",
        ].join(" ")}
      >
        <input
          ref={inputRef}
          type="file"
          accept="video/*"
          className="hidden"
          onChange={handleInputChange}
        />

        {file ? (
          <div className="flex items-center gap-3 text-sm">
            <HugeiconsIcon
              icon={Upload01Icon}
              strokeWidth={2}
              className="size-5 shrink-0 text-primary"
            />
            <div className="flex flex-col">
              <span className="font-medium">{file.name}</span>
              <span className="text-muted-foreground">
                {formatBytes(file.size)}
              </span>
            </div>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onReset();
              }}
              className="ml-2 rounded-full p-1 hover:bg-muted"
              disabled={disabled}
            >
              <HugeiconsIcon
                icon={Cancel01Icon}
                strokeWidth={2}
                className="size-4"
              />
            </button>
          </div>
        ) : (
          <>
            <HugeiconsIcon
              icon={Upload01Icon}
              strokeWidth={1.5}
              className="size-10 text-muted-foreground"
            />
            <div className="text-center">
              <p className="font-medium text-sm">Drop a video here</p>
              <p className="text-muted-foreground text-xs mt-1">
                or click to browse
              </p>
            </div>
          </>
        )}
      </div>

      {error && <p className="text-destructive text-sm">{error}</p>}
    </div>
  );
}
