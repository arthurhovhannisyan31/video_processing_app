"use client";

import type { ApiError } from "configs/types";
import { useEffect, useMemo, useState } from "react";
import { type SubmitHandler, useForm } from "react-hook-form";

import { zodResolver } from "@hookform/resolvers/zod";
import { HttpStatusCode } from "axios";
import { getPasswordStrength } from "components/modules/auth/common/constants";
import { PasswordStrength } from "components/modules/auth/common/password-strength";
import {
  type ResetPasswordSchema,
  resetPasswordSchema,
  TOKEN_QUERY_PARAM,
} from "components/modules/auth/reset-password-form/constants";
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
import { PasswordInput } from "components/ui/password-input";
import { RootPath } from "configs/routes/constants";
import { resetPassword } from "generated/client";
import { getErrorMessage } from "helpers/utils";
import { useAtom } from "jotai";
import { cn } from "lib/utils";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { toast } from "sonner";
import { userStore } from "store/user";

export function ResetPasswordForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const [isLoading, setIsLoading] = useState(false);
  const [token, setToken] = useAtom(userStore.resetToken);
  const [isLinkInvalid, setIsLinkInvalid] = useState(false);
  const router = useRouter();

  const {
    register,
    handleSubmit,
    formState: { errors, dirtyFields, touchedFields },
    watch,
    setError,
  } = useForm<ResetPasswordSchema>({
    defaultValues: {
      password: "",
      confirmPassword: "",
    },
    mode: "onTouched",
    resolver: zodResolver(resetPasswordSchema),
  });

  const passwordValue = watch("password");
  const passwordStrengths = useMemo(
    () => getPasswordStrength(passwordValue),
    [passwordValue],
  );

  useEffect(() => {
    if (token) return;

    const searchParams = new URLSearchParams(window.location.search);
    const tokenParam = searchParams.get(TOKEN_QUERY_PARAM);

    // Remove the token from the address bar and browser history
    if (tokenParam) {
      setToken(tokenParam);

      searchParams.delete(TOKEN_QUERY_PARAM);
      const search = searchParams.toString();
      window.history.replaceState(
        null,
        "",
        `${window.location.pathname}${search ? `?${search}` : ""}`,
      );
    }
  }, [token, setToken]);

  const onSubmit: SubmitHandler<ResetPasswordSchema> = async ({ password }) => {
    if (!token) {
      return;
    }
    if (!passwordStrengths.allValid) {
      setError("password", {
        type: "manual",
        message: "Password is not strong enough",
      });

      return;
    }

    setIsLoading(true);
    try {
      const response = await resetPassword({ body: { token, password } });

      if (response.error) {
        throw response;
      }

      toast.success("Password updated. Please log in with your new password");
      router.push(`/${RootPath.SignIn}`);
    } catch (err) {
      const error = err as ApiError;
      const message = getErrorMessage(error);

      // Password requirements are validated on the client,
      // so a token related 400 means the link can't be used anymore
      if (
        error.status === HttpStatusCode.BadRequest &&
        message?.toLowerCase().includes("token")
      ) {
        setIsLinkInvalid(true);
      } else {
        toast.error(
          message || getErrorMessage(error, "Failed to reset password"),
        );
      }
      setIsLoading(false);
    }
  };

  if (token === undefined) {
    return null;
  }

  if (!token || isLinkInvalid) {
    return (
      <div className={cn("flex flex-col gap-6", className)} {...props}>
        <Card>
          <CardHeader>
            <CardTitle>Reset link is invalid</CardTitle>
            <CardDescription>
              This password reset link is invalid or has expired. Reset links
              can be used only once.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <FieldGroup>
              <Field>
                <Button
                  onClick={() => router.push(`/${RootPath.ForgotPassword}`)}
                >
                  Request a new link
                </Button>
                <FieldDescription className="text-center">
                  <Link href={`/${RootPath.SignIn}`}>Back to login</Link>
                </FieldDescription>
              </Field>
            </FieldGroup>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <Card>
        <CardHeader>
          <CardTitle>Set a new password</CardTitle>
          <CardDescription>
            Choose a new password for your account
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit(onSubmit)}>
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="password">New Password</FieldLabel>
                <PasswordInput
                  id="password"
                  required
                  isDirty={!!dirtyFields.password}
                  autoComplete="new-password"
                  {...register("password")}
                />
                <FieldError errors={[errors.password]} />
              </Field>
              <Field>
                <PasswordStrength
                  hasDigits={passwordStrengths.hasDigits}
                  hasLowercase={passwordStrengths.hasLowercase}
                  hasSpecial={passwordStrengths.hasSpecial}
                  hasUppercase={passwordStrengths.hasUppercase}
                  touched={touchedFields.password}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="confirm-password">
                  Confirm Password
                </FieldLabel>
                <PasswordInput
                  id="confirm-password"
                  required
                  isDirty={!!dirtyFields.confirmPassword}
                  autoComplete="new-password"
                  {...register("confirmPassword")}
                />
                <FieldError errors={[errors.confirmPassword]} />
              </Field>
              <Field>
                <Button disabled={isLoading} type="submit">
                  Reset password
                </Button>
                <FieldDescription className="text-center">
                  <Link href={`/${RootPath.SignIn}`}>Back to login</Link>
                </FieldDescription>
              </Field>
            </FieldGroup>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
