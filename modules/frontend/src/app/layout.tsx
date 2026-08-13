import type { Metadata } from "next";
import type { ReactNode } from "react";

import { Geist, Geist_Mono, Inter } from "next/font/google";

import "./globals.css";
import "lib/setup";

import Providers from "app/providers";
import { cn } from "lib/utils";

const inter = Inter({ subsets: ["latin"], variable: "--font-sans" });

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Open cut App",
  description: "Open cut application",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={cn(
        geistSans.variable,
        geistMono.variable,
        "font-sans",
        inter.variable,
      )}
    >
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
