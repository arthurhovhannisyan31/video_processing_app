"use client";

import { Button } from "components/ui/button";
import { Card, CardContent } from "components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "components/ui/collapsible";
import { Skeleton } from "components/ui/skeleton";
import { ChevronDownIcon } from "lucide-react";

interface VideoInspectResultProps {
  data: Record<string, unknown> | null;
  isLoading: boolean;
}

export function VideoInspectResult({
  data,
  isLoading,
}: VideoInspectResultProps) {
  if (isLoading) {
    return (
      <div className="flex flex-col gap-2">
        <Skeleton className="h-12 w-full" />
      </div>
    );
  }

  if (!data) return null;

  return (
    <Card className="mx-auto w-full p-0">
      <CardContent className={"p-0"}>
        <Collapsible className="rounded-md data-open:bg-muted">
          <CollapsibleTrigger
            render={
              <Button variant="ghost" className="w-full text-base h-10">
                Inspection results
                <ChevronDownIcon className="ml-auto group-data-panel-open/button:rotate-180" />
              </Button>
            }
          />
          <CollapsibleContent className="flex flex-col items-start p-2.5 text-sm">
            <div className="flex flex-col gap-2 w-full">
              <pre className="overflow-auto rounded-lg border bg-muted p-4 font-mono text-sm leading-relaxed max-h-96">
                {JSON.stringify(data, null, 2)}
              </pre>
            </div>
          </CollapsibleContent>
        </Collapsible>
      </CardContent>
    </Card>
  );
}
