"use client";

import { useMutation } from "@tanstack/react-query";
import { Button } from "components/ui/button";
import { processVideo } from "generated/client/sdk.gen";

interface VideoCompressProps {
  file: File | null;
  isInspecting: boolean;
}

export function VideoCompress({ file, isInspecting }: VideoCompressProps) {
  const { mutate, isPending, error, reset } = useMutation({
    mutationFn: async (f: File) => {
      const res = await processVideo({
        body: { video: f, operation: "compress" },
        responseType: "blob",
      });
      return res.data as unknown as Blob;
    },
    onSuccess(blob, f) {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      const stem = f.name.replace(/\.[^.]+$/, "");
      a.href = url;
      a.download = `${stem}_compressed.mp4`;
      a.click();
      URL.revokeObjectURL(url);
    },
  });

  const disabled = !file || isInspecting || isPending;
  const errorMessage =
    error instanceof Error
      ? error.message
      : error
        ? "Compression failed."
        : null;

  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm font-medium text-muted-foreground">Actions</p>
      <Button
        onClick={() => {
          reset();
          if (file) mutate(file);
        }}
        disabled={disabled}
        className="w-full sm:w-auto"
      >
        {isPending ? "Compressing…" : "Compress"}
      </Button>
      {errorMessage && (
        <p className="text-destructive text-sm">{errorMessage}</p>
      )}
    </div>
  );
}
