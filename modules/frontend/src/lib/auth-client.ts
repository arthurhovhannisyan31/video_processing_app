import type { AuthResponse } from "generated/client";
import type { AuthSession } from "lib/auth";
import { useMemo } from "react";

import { createAuthClient } from "better-auth/react";

export const authClient = createAuthClient();

export const getAuthData = async (): Promise<AuthResponse | undefined> => {
  const { data, error } = await authClient.getSession();

  if (error) {
    return;
  }
  const sessionData = data as AuthSession;

  if ("data" in sessionData.user) {
    return sessionData.user.data as AuthResponse;
  }

  return;
};

export const useAuthData = (): AuthResponse | undefined => {
  const { data: session, isPending } = authClient.useSession();

  return useMemo(() => {
    const sessionData = session as AuthSession;

    if (!isPending && sessionData) {
      if ("data" in sessionData.user) {
        return sessionData.user.data as AuthResponse;
      }
    }
  }, [isPending, session]);
};
