import type { TableRowElement } from "components/modules/video/processing-summary/types";
import type { ProcessingOptions } from "components/modules/video/types";
import { type FC, useMemo } from "react";

import {
  Codec,
  getOptionLabel,
  Operation,
  presetOptions,
  qualityOptions,
} from "components/modules/video/constants";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "components/ui/table";
import { formatBytesToMB } from "lib/utils";

const EMPTY_VALUE = "-";

const formatSizeChange = (originalSize: number, processedSize: number) => {
  if (!originalSize) return EMPTY_VALUE;

  const change = ((processedSize - originalSize) / originalSize) * 100;
  const sign = change > 0 ? "+" : "";

  return `${sign}${change.toFixed(1)}%`;
};

const formatDuration = (ms?: number) => {
  if (ms === undefined) return EMPTY_VALUE;
  if (ms < 1000) return `${Math.round(ms)} ms`;

  const seconds = ms / 1000;
  if (seconds < 60) return `${seconds.toFixed(1)} s`;

  const minutes = Math.floor(seconds / 60);
  const restSeconds = Math.round(seconds % 60);

  return `${minutes}m ${restSeconds.toString().padStart(2, "0")}s`;
};

export interface ProcessingSummaryProps {
  originalSize: number;
  processedSize?: number;
  appliedOptions?: ProcessingOptions;
  processingTimeMs?: number;
}

export const ProcessingSummary: FC<ProcessingSummaryProps> = ({
  originalSize,
  processedSize,
  appliedOptions,
  processingTimeMs,
}) => {
  const isProcessed = processedSize !== undefined;

  const rows = useMemo<TableRowElement[]>(() => {
    const temp: TableRowElement[] = [
      { label: "Original size", value: formatBytesToMB(originalSize, 2) },
    ];
    if (isProcessed) {
      temp.push(
        {
          label: "Processed size",
          value: isProcessed ? formatBytesToMB(processedSize, 2) : EMPTY_VALUE,
        },
        {
          label: "Size change",
          value: isProcessed
            ? formatSizeChange(originalSize, processedSize)
            : EMPTY_VALUE,
        },
        {
          label: "Processing time",
          value: formatDuration(processingTimeMs),
        },
        {
          label: "Operation",
          value: isProcessed
            ? `${Operation.Compress} (${Codec.X264})`
            : EMPTY_VALUE,
        },
        {
          label: "Preset",
          value: appliedOptions
            ? `${getOptionLabel(presetOptions, appliedOptions.preset)} (${appliedOptions.preset})`
            : EMPTY_VALUE,
        },
        {
          label: "Quality",
          value: appliedOptions
            ? `${getOptionLabel(qualityOptions, appliedOptions.quality)} (CRF ${appliedOptions.quality})`
            : EMPTY_VALUE,
        },
      );
    }

    return temp;
  }, [
    appliedOptions,
    isProcessed,
    originalSize,
    processedSize,
    processingTimeMs,
  ]);

  return (
    <div className={"rounded-md border"}>
      <Table className={"text-sm"}>
        <TableHeader>
          <TableRow>
            <TableHead>Property</TableHead>
            <TableHead>Value</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map(({ label, value }) => (
            <TableRow key={label}>
              <TableCell className={"text-muted-foreground"}>{label}</TableCell>
              <TableCell>{value}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
};
