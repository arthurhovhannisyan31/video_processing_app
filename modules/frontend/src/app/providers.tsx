"use client";

import type { PropsWithChildren } from "react";

import { Analytics } from "@vercel/analytics/react";
import { SpeedInsights } from "@vercel/speed-insights/next";
import { ThemeProvider } from "components/theme-provider";
import { Toaster } from "components/ui/sonner";
import { TooltipProvider } from "components/ui/tooltip";
import "lib/setup";

import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "lib/query-client";

export default function Providers({ children }: PropsWithChildren) {
  return (
    <QueryClientProvider client={queryClient}>
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
      <Analytics />
      <SpeedInsights />
    </QueryClientProvider>
  );
}
