"use client";

import {
  type ChangeEvent,
  type DragEvent,
  useCallback,
  useMemo,
  useRef,
} from "react";

import { Upload01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import {
  getErrorsDict,
  MAX_FILES_COUNT,
} from "components/modules/video/constants";
import {
  getMaxBodySize,
  validate_file,
} from "components/modules/video/helpers";
import { useHoverEvent } from "hooks/useHoverEvent";
import { toast } from "sonner";

interface VideoDropZoneProps {
  addFiles: (files: File[]) => void;
}

export function DropZone({ addFiles }: VideoDropZoneProps) {
  const { isOver, ref } = useHoverEvent();

  const inputRef = useRef<HTMLInputElement>(null);

  const showAlert = useCallback((msg: string) => {
    toast.error(msg);
  }, []);
  const maxBodySize = useMemo(() => getMaxBodySize(), []);
  const errorsDict = useMemo(() => getErrorsDict(maxBodySize), [maxBodySize]);

  const processFiles = async (files: FileList) => {
    let count = 0;

    if (files.length > MAX_FILES_COUNT) {
      toast.warning(`Files processing limit is ${MAX_FILES_COUNT}.`);
    }
    const validFiles: File[] = [];

    for (const file of files) {
      if (!validate_file(file, errorsDict, showAlert)) {
        continue;
      }
      count++;

      if (count > MAX_FILES_COUNT) {
        toast.warning(`Only first ${count - 1} validated files were uploaded.`);
        break;
      }

      validFiles.push(file);
    }

    addFiles(validFiles);
  };

  const handleDrop = async (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    await processFiles(e.dataTransfer.files);
  };

  const openFileUploadDialog = () => {
    if (inputRef.current) {
      inputRef.current.value = "";
      inputRef.current?.click();
    }
  };

  const handleInputChange = async (e: ChangeEvent<HTMLInputElement>) => {
    if (inputRef.current?.files?.length) {
      await processFiles(inputRef.current.files);
      e.target.value = "";
    }
  };

  const preventDragEvent = (e: DragEvent<HTMLInputElement>): void => {
    e.preventDefault();
  };

  return (
    <div ref={ref} className="flex flex-col gap-2 h-full">
      {/* biome-ignore lint/a11y/useKeyWithClickEvents: not relevant */}
      {/** biome-ignore lint/a11y/noStaticElementInteractions: not relevant */}
      <div
        onDrop={handleDrop}
        onClick={openFileUploadDialog}
        onDragEnter={preventDragEvent}
        onDragOver={preventDragEvent}
        className={[
          "flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed p-10",
          "transition-colors cursor-pointer hover:border-primary/60 hover:bg-muted/40 h-full",
          isOver ? "border-primary bg-primary/5" : "border-border",
        ].join(" ")}
      >
        <input
          ref={inputRef}
          type="file"
          accept="video/*"
          className="hidden"
          onChange={handleInputChange}
          multiple={true}
        />
        <HugeiconsIcon
          icon={Upload01Icon}
          strokeWidth={1.5}
          className="size-10 text-muted-foreground"
        />
        <div className="text-center">
          <p className="font-medium text-sm">Drop your files here</p>
          <p className="text-muted-foreground text-xs mt-1">
            or click to browse
          </p>
          <p className="text-muted-foreground text-xs mt-1">5 files max</p>
        </div>
      </div>
    </div>
  );
}
