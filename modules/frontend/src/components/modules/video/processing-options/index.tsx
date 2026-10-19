import type { ProcessingOptions } from "components/modules/video/types";
import { type FC, useId } from "react";

import {
  type Preset,
  presetOptions,
  type Quality,
  qualityOptions,
  type SelectOption,
} from "components/modules/video/constants";
import { Label } from "components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "components/ui/select";

interface OptionSelectProps<T extends string> {
  label: string;
  value: T;
  options: SelectOption<T>[];
  disabled?: boolean;
  onChange: (value: T) => void;
}

const OptionSelect = <T extends string>({
  label,
  value,
  options,
  disabled,
  onChange,
}: OptionSelectProps<T>) => {
  const id = useId();

  return (
    <div className={"flex items-center gap-2"}>
      <Label htmlFor={id} className={"text-sm text-muted-foreground"}>
        {label}
      </Label>
      <Select<T>
        value={value}
        items={options}
        disabled={disabled}
        onValueChange={(newValue) => {
          if (newValue) {
            onChange(newValue);
          }
        }}
      >
        <SelectTrigger id={id} className={"h-8 min-w-36 text-sm"}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem
              key={option.value}
              value={option.value}
              className={"text-sm"}
            >
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
};

export interface ProcessingOptionsSelectProps {
  options: ProcessingOptions;
  disabled?: boolean;
  onChange: (options: ProcessingOptions) => void;
}

export const ProcessingOptionsSelect: FC<ProcessingOptionsSelectProps> = ({
  options,
  disabled,
  onChange,
}) => (
  <div className={"flex flex-wrap gap-4 px-2"}>
    <OptionSelect<Preset>
      label={"Preset"}
      value={options.preset}
      options={presetOptions}
      disabled={disabled}
      onChange={(preset) => onChange({ ...options, preset })}
    />
    <OptionSelect<Quality>
      label={"Quality"}
      value={options.quality}
      options={qualityOptions}
      disabled={disabled}
      onChange={(quality) => onChange({ ...options, quality })}
    />
  </div>
);
