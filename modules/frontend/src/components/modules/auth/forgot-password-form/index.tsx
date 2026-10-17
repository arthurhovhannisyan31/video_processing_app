"use client";

import type { ApiError } from "configs/types";
import { useEffect, useState } from "react";
import { type SubmitHandler, useForm } from "react-hook-form";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  PASSWORD_RESET_COOLDOWN_SEC,
  PASSWORD_RESET_TTL_MIN,
} from "components/modules/auth/common/constants";
import {
  type ForgotPasswordSchema,
  forgotPasswordSchema,
} from "components/modules/auth/forgot-password-form/constants";
import { Button } from "components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "components/ui/card";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "components/ui/field";
import { Input } from "components/ui/input";
import { RootPath } from "configs/routes/constants";
import { forgotPassword } from "generated/client";
import { getErrorMessage } from "helpers/utils";
import { cn } from "lib/utils";
import Link from "next/link";
import { toast } from "sonner";

export function ForgotPasswordForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const [isLoading, setIsLoading] = useState(false);
  const [sentTo, setSentTo] = useState<string>();
  const [cooldown, setCooldown] = useState(0);

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<ForgotPasswordSchema>({
    defaultValues: {
      email: "",
    },
    mode: "onTouched",
    resolver: zodResolver(forgotPasswordSchema),
  });

  useEffect(() => {
    if (cooldown <= 0) {
      return;
    }
    const timeout = setTimeout(() => setCooldown(cooldown - 1), 1000);

    return () => clearTimeout(timeout);
  }, [cooldown]);

  const requestReset = async (email: string) => {
    setIsLoading(true);
    try {
      const response = await forgotPassword({ body: { email } });

      if (response.error) {
        throw response;
      }

      setSentTo(email);
      setCooldown(PASSWORD_RESET_COOLDOWN_SEC);
    } catch (err) {
      toast.error(
        getErrorMessage(err as ApiError, "Failed to request password reset"),
      );
    } finally {
      setIsLoading(false);
    }
  };

  const onSubmit: SubmitHandler<ForgotPasswordSchema> = async ({ email }) => {
    await requestReset(email);
  };

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <Card>
        <CardHeader>
          <CardTitle>
            {sentTo ? "Check your email" : "Forgot your password?"}
          </CardTitle>
          <CardDescription>
            {sentTo ? (
              <>
                If an account exists for <b>{sentTo}</b>, we&apos;ve sent a link
                to reset your password. The link expires in{" "}
                {PASSWORD_RESET_TTL_MIN} minutes.
              </>
            ) : (
              "Enter your email and we'll send you a link to reset your password"
            )}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {sentTo ? (
            <FieldGroup>
              <Field>
                <Button
                  variant="outline"
                  disabled={isLoading || cooldown > 0}
                  onClick={() => requestReset(sentTo)}
                >
                  {cooldown > 0 ? `Resend in ${cooldown}s` : "Resend email"}
                </Button>
                <FieldDescription className="text-center">
                  <Link href={`/${RootPath.SignIn}`}>Back to login</Link>
                </FieldDescription>
              </Field>
            </FieldGroup>
          ) : (
            <form onSubmit={handleSubmit(onSubmit)}>
              <FieldGroup>
                <Field>
                  <FieldLabel htmlFor="email">Email</FieldLabel>
                  <Input
                    id="email"
                    type="email"
                    placeholder="m@example.com"
                    autoComplete="email"
                    required
                    {...register("email")}
                  />
                  <FieldError errors={[errors.email]} />
                </Field>
                <Field>
                  <Button disabled={isLoading} type="submit">
                    Send reset link
                  </Button>
                  <FieldDescription className="text-center">
                    Remember your password?{" "}
                    <Link href={`/${RootPath.SignIn}`}>Back to login</Link>
                  </FieldDescription>
                </Field>
              </FieldGroup>
            </form>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
