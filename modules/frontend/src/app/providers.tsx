"use client";

import type { ReactNode } from "react";

import { ThemeProvider } from "components/theme-provider";
import { Toaster } from "components/ui/sonner";
import { TooltipProvider } from "components/ui/tooltip";
import "lib/setup";

export default function Providers({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
    >
      <TooltipProvider>
        {children}
        <Toaster />
      </TooltipProvider>
    </ThemeProvider>
  );
}
