import type { MetadataRoute } from "next";

import { SITE_URL } from "configs/constants";
import { RootPath } from "configs/routes/constants";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: "*",
      allow: "/",
      disallow: [
        `/${RootPath.ForgotPassword}`,
        `/${RootPath.ResetPassword}`,
        "/api/",
        "/monitoring",
      ],
    },
    sitemap: `${SITE_URL}/sitemap.xml`,
  };
}
