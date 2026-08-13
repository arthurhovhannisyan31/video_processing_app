import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
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

export default nextConfig;
