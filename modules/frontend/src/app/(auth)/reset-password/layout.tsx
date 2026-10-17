import type { Metadata } from "next";
import type { PropsWithChildren } from "react";

import { ModeToggle } from "components/theme-mode-toggle";

export const metadata: Metadata = {
  title: "Reset password",
  // Prevent leaking the reset token from the URL via Referer header
  referrer: "no-referrer",
};

export default async function ResetPasswordLayout({
  children,
}: PropsWithChildren) {
  return (
    <section className={"relative"}>
      <div className={"absolute right-5 top-5"}>
        <ModeToggle />
      </div>
      <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
        <div className="w-full max-w-sm">{children}</div>
      </div>
    </section>
  );
}
