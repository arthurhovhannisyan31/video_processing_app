import type { NextConfig } from "next";

import {
  type SentryBuildOptions,
  withSentryConfig,
} from "@sentry/nextjs/config";

const nextConfig: NextConfig = {
  reactCompiler: true,
  experimental: {
    turbopackFileSystemCacheForDev: false, // Disables Turbopack's dev cache
  },
  async redirects() {
    return [
      {
        source: "/",
        destination: "/video",
        permanent: true,
      },
    ];
  },
};

const sentryBuildOptions: SentryBuildOptions = {
  org: process.env.SENTRY_ORG_NAME,
  project: process.env.SENTRY_PROJECT_NAME,
  silent: !process.env.CI,
  widenClientFileUpload: true,
  tunnelRoute: "/monitoring",
  webpack: {
    automaticVercelMonitors: true,
    treeshake: {
      removeDebugLogging: true,
    },
  },
};

export default withSentryConfig(nextConfig, sentryBuildOptions);
