import * as Sentry from "@sentry/nextjs";
import { SENTRY_DSN } from "configs/constants";

Sentry.init({
  // Skip local prod builds
  enabled:
    process.env.NODE_ENV === "production" &&
    process.env.NEXT_PUBLIC_LOCAL_ENV !== "true",
  dsn: SENTRY_DSN,
  tracesSampleRate: process.env.NODE_ENV === "production" ? 0.1 : 1.0,
});
