import type { FC } from "react";

import { Tooltip, TooltipContent, TooltipTrigger } from "components/ui/tooltip";
import { cn } from "lib/utils";
import { Check } from "lucide-react";

export interface PasswordStrengthProps extends React.ComponentProps<"div"> {
  hasUppercase?: boolean;
  hasLowercase?: boolean;
  hasDigits?: boolean;
  hasSpecial?: boolean;
  touched?: boolean;
}

export const PasswordStrength: FC<PasswordStrengthProps> = ({
  hasDigits,
  hasLowercase,
  hasSpecial,
  hasUppercase,
  touched,
  className,
  ...props
}) => {
  return (
    <div
      data-slot="field-content"
      className={cn(
        "group/field-content flex flex-row justify-items-start gap-3 leading-snug",
        className,
      )}
      {...props}
    >
      <div className={"flex flex-row gap-1 items-center"}>
        <Check
          className={`${hasUppercase ? "text-green-500" : "text-transparent"} w-4 h-4`}
        />
        <Tooltip>
          <TooltipTrigger render={<p>A-Z</p>} />
          <TooltipContent>
            <p>Uppercase characters A-Z</p>
          </TooltipContent>
        </Tooltip>
      </div>
      <div className={"flex flex-row gap-1 items-center"}>
        <Check
          className={`${hasLowercase ? "text-green-500" : "text-transparent"} w-4 h-4`}
        />
        <Tooltip>
          <TooltipTrigger render={<p>a-z</p>} />
          <TooltipContent>
            <p>Lowercase characters a-z</p>
          </TooltipContent>
        </Tooltip>
      </div>
      <div className={"flex flex-row gap-1 items-center"}>
        <Check
          className={`${hasDigits ? "text-green-500" : "text-transparent"} w-4 h-4`}
        />
        <Tooltip>
          <TooltipTrigger render={<span>0-9</span>} />
          <TooltipContent>
            <p>Digits 0-9</p>
          </TooltipContent>
        </Tooltip>
      </div>
      <div className={"flex flex-row gap-1 items-center"}>
        <Check
          className={`${hasSpecial ? "text-green-500" : "text-transparent"} w-4 h-4`}
        />
        <Tooltip>
          <TooltipTrigger render={<span>!,$,#,%...</span>} />
          <TooltipContent>
            <p>Special characters (!,$,#,%, etc.)</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
};
