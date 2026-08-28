"use client";

import type { ApiError } from "configs/types";
import type { AbortControllerResult } from "lib/hooks/useAbortController";

import { useMutation } from "@tanstack/react-query";
import { Button } from "components/ui/button";
import { Spinner } from "components/ui/spinner";
import { processVideo } from "generated/client/sdk.gen";

interface VideoCompressProps {
  file: File | null;
  isInspecting: boolean;
  abortController: AbortControllerResult;
  setError: (val: string | null) => void;
}

export function VideoCompress({
  file,
  isInspecting,
  abortController,
  setError,
}: VideoCompressProps) {
  const { mutate, isPending, error, reset } = useMutation({
    mutationFn: async (f: File) => {
      abortController.abort();
      abortController.init();

      const res = await processVideo({
        body: { video: f, operation: "compress" },
        responseType: "blob",
        signal: abortController.ref.current?.signal,
      });
      return res.data as unknown as Blob;
    },
    onSuccess(blob, f) {
      setError(null);

      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      const stem = f.name.replace(/\.[^.]+$/, "");
      a.href = url;
      a.download = `${stem}_compressed.mp4`;
      a.click();
      URL.revokeObjectURL(url);
    },
    onError: (err) => {
      const error = err as ApiError;
      const errorMessage =
        error.error || error.message || error.status || "Processing failed.";

      setError(errorMessage.toString());
    },
    onSettled: () => {
      abortController.ref.current = null;
    },
  });

  const disabled = !file || isInspecting || isPending;
  const errorMessage =
    error instanceof Error
      ? error.message
      : error
        ? "Compression failed."
        : null;

  const handleCompress = () => {
    if (isPending) {
      abortController.abort();

      return;
    }

    reset();
    if (file) mutate(file);
  };

  return (
    <div className="flex flex-col gap-3">
      <Button
        onClick={handleCompress}
        disabled={disabled}
        className="w-full sm:w-auto h-10"
      >
        {isPending ? <Spinner /> : "Compress"}
      </Button>
      {errorMessage && (
        <p className="text-destructive text-sm">{errorMessage}</p>
      )}
    </div>
  );
}
