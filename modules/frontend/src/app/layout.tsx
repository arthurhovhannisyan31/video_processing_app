import type { Metadata } from "next";
import type { PropsWithChildren } from "react";

import { Inter } from "next/font/google";

import "./globals.css";
import "lib/setup";

import Providers from "app/providers";
import { SITE_DESCRIPTION, SITE_NAME, SITE_URL } from "configs/constants";
import { cn } from "lib/utils";

const inter = Inter({ subsets: ["latin"], variable: "--font-sans" });

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: `${SITE_NAME} – compress & convert videos online`,
    template: `%s | ${SITE_NAME}`,
  },
  description: SITE_DESCRIPTION,
  applicationName: SITE_NAME,
  keywords: [
    "video compressor",
    "compress video online",
    "video converter",
    "mp4 to webm",
    "mp4 to mkv",
    "mp4 to mov",
    "reduce video size",
    "ffmpeg online",
  ],
  openGraph: {
    type: "website",
    siteName: SITE_NAME,
    locale: "en_US",
    url: "/",
    title: `${SITE_NAME} – compress & convert videos online`,
    description: SITE_DESCRIPTION,
  },
  twitter: {
    card: "summary_large_image",
    title: `${SITE_NAME} – compress & convert videos online`,
    description: SITE_DESCRIPTION,
  },
};

export default function RootLayout({ children }: PropsWithChildren) {
  return (
    <html lang="en" className={cn("font-sans", inter.variable)}>
      <body className={"h-full"}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
