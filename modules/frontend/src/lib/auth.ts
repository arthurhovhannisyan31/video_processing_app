import type { AuthResponse } from "generated/client";

import { type BetterAuthPlugin, betterAuth } from "better-auth";
import { createAuthEndpoint } from "better-auth/api";
import { deleteSessionCookie, setSessionCookie } from "better-auth/cookies";
import { BETTER_AUTH_SECRET } from "configs/constants";
import zod from "zod";

const authResponseSchema = zod.object({
  user: zod.object({
    user_id: zod.string(),
    email: zod.string().email(),
    username: zod.string(),
  }),
  token: zod.string(),
});

export const auth = betterAuth({
  secret: BETTER_AUTH_SECRET,
  emailAndPassword: {
    enabled: true,
  },
  session: {
    expiresIn: 60 * 60 * 24 * 7, // 7 days
    cookieCache: {
      strategy: "jwt",
      enabled: true,
      maxAge: 60 * 60 * 24 * 7, // 7 days
    },
  },
  plugins: [
    {
      id: "custom-auth",
      endpoints: {
        customAuth: createAuthEndpoint(
          "/custom-auth",
          {
            method: "POST",
            body: authResponseSchema,
          },
          async (ctx) => {
            const { user, token } = ctx.body as AuthResponse;
            const userId = user.user_id;

            const session = await ctx.context.internalAdapter.createSession(
              user.user_id,
              false,
              {
                token,
                userId,
              },
            );

            await setSessionCookie(ctx, {
              session,
              user: {
                id: user.user_id,
                email: user.email,
                emailVerified: true,
                createdAt: new Date(),
                updatedAt: new Date(),
                name: ctx.body.token,
                // @ts-expect-error Populate auth response data in user data
                data: ctx.body,
              },
            });

            return ctx.json(ctx.body);
          },
        ),
        customSignOut: createAuthEndpoint(
          "/custom-sign-out",
          { method: "POST" },
          async (ctx) => {
            deleteSessionCookie(ctx);

            return ctx.json({
              success: true,
            });
          },
        ),
      },
    } satisfies BetterAuthPlugin,
  ],
});

export type AuthSession = typeof auth.$Infer.Session;
