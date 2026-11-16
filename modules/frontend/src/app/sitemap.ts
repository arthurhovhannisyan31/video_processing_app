import type { MetadataRoute } from "next";

import { SITE_URL } from "configs/constants";
import { RootPath } from "configs/routes/constants";

export default function sitemap(): MetadataRoute.Sitemap {
  const lastModified = new Date();

  return [
    { url: SITE_URL, lastModified, changeFrequency: "monthly", priority: 1 },
    {
      url: `${SITE_URL}/${RootPath.Signup}`,
      lastModified,
      changeFrequency: "yearly",
      priority: 0.5,
    },
    {
      url: `${SITE_URL}/${RootPath.SignIn}`,
      lastModified,
      changeFrequency: "yearly",
      priority: 0.3,
    },
  ];
}
