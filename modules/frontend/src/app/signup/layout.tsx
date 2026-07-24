import type { Metadata } from "next";
import type { PropsWithChildren } from "react";

import { ModeToggle } from "components/theme-mode-toggle";

export const metadata: Metadata = {
  title: "Sign up",
};

export default async function SignupLayout({ children }: PropsWithChildren) {
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
