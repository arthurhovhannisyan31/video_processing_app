"use client";

import { Skeleton } from "components/ui/skeleton";

interface VideoInspectResultProps {
  data: Record<string, unknown> | null;
  isLoading: boolean;
  error: string | null;
}

export function VideoInspectResult({
  data,
  isLoading,
  error,
}: VideoInspectResultProps) {
  if (isLoading) {
    return (
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-48 w-full rounded-lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg border border-destructive/30 bg-destructive/10 p-4">
        <p className="text-destructive text-sm">{error}</p>
      </div>
    );
  }

  if (!data) return null;

  return (
    <div className="flex flex-col gap-2">
      <p className="text-sm font-medium text-muted-foreground">
        Inspection result
      </p>
      <pre className="overflow-auto rounded-lg border bg-muted p-4 font-mono text-xs leading-relaxed max-h-96">
        {JSON.stringify(data, null, 2)}
      </pre>
    </div>
  );
}
