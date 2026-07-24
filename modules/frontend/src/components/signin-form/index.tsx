"use client";

import { useState } from "react";
import { type SubmitHandler, useForm } from "react-hook-form";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  type SignInSchema,
  signInSchema,
} from "components/signin-form/constants";
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
import { PasswordInput } from "components/ui/password-input";
import { RootPath } from "configs/routes/constants";
import { authClient } from "lib/auth-client";
import { cn } from "lib/utils";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { toast } from "sonner";

export function SignInForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();

  const {
    register,
    handleSubmit,
    formState: { errors, dirtyFields },
  } = useForm<SignInSchema>({
    defaultValues: {
      email: "",
      password: "",
    },
    mode: "onTouched",
    resolver: zodResolver(signInSchema),
  });

  const onSubmit: SubmitHandler<SignInSchema> = async ({ email, password }) => {
    await authClient.signIn.email(
      {
        email,
        password,
        callbackURL: "/",
      },
      {
        onRequest: () => {
          setIsLoading(true);
        },
        onSuccess: () => {
          router.push("/");
        },
        onError: (ctx) => {
          setIsLoading(false);
          toast.error(
            (ctx.error.error.message || ctx.response.statusText).toString(),
          );
        },
      },
    );
  };

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <Card>
        <CardHeader>
          <CardTitle>Login to your account</CardTitle>
          <CardDescription>
            Enter your email below to login to your account
          </CardDescription>
        </CardHeader>
        <CardContent>
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
                <div className="flex items-center">
                  <FieldLabel htmlFor="password">Password</FieldLabel>
                </div>
                <PasswordInput
                  id="password"
                  required
                  isDirty={!!dirtyFields.password}
                  autoComplete="current-password"
                  {...register("password")}
                />
                <FieldError errors={[errors.password]} />
              </Field>
              <Field>
                <Button disabled={isLoading} type="submit">
                  Login
                </Button>
                <FieldDescription className="text-center">
                  Don&apos;t have an account?{" "}
                  <Link href={RootPath.Signup}>Sign up</Link>
                </FieldDescription>
              </Field>
            </FieldGroup>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
