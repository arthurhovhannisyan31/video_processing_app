import * as Sentry from "@sentry/nextjs";
import { SENTRY_DSN } from "configs/constants";

Sentry.init({
  // Skip local prod builds
  enabled:
    process.env.NODE_ENV === "production" &&
    process.env.NEXT_PUBLIC_LOCAL_ENV !== "true",
  dsn: SENTRY_DSN,
  integrations: [
    Sentry.replayIntegration({
      maskAllInputs: true,
    }),
  ],
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,
});

export const onRouterTransitionStart = Sentry.captureRouterTransitionStart;
