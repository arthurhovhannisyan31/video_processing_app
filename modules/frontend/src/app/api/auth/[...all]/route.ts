import type { AuthResponse } from "generated/client";

import axios, { type AxiosResponse } from "axios";
import { toNextJsHandler } from "better-auth/next-js";
import { NEXT_PUBLIC_BACKEND_BASE_URL } from "configs/constants";
import { auth } from "lib/auth";
import { type NextRequest, NextResponse } from "next/server";

const nativeHandlers = toNextJsHandler(auth);

export type AxiosAuthResponse = AxiosResponse<
  AuthResponse,
  AnyObject,
  AnyObject
>;

export async function POST(request: NextRequest) {
  const pathname = request.nextUrl.pathname;

  try {
    if (pathname.endsWith("sign-in/email")) {
      const { email, password } = await request.json();

      const response: AxiosAuthResponse = await axios.post<AuthResponse>(
        `${NEXT_PUBLIC_BACKEND_BASE_URL}/auth/login`,
        {
          email,
          password,
        },
      );

      return handleAuthResponse(response);
    }
    if (pathname.endsWith("sign-up/email")) {
      const { email, password, name } = await request.json();

      const response: AxiosAuthResponse = await axios.post<AuthResponse>(
        `${NEXT_PUBLIC_BACKEND_BASE_URL}/auth/register`,
        {
          email,
          password,
          username: name,
        },
      );

      return handleAuthResponse(response);
    }
    if (pathname.endsWith("sign-out")) {
      const pluginResponse = await auth.api.customSignOut({
        headers: request.headers,
        asResponse: true,
      });

      return NextResponse.json(
        { message: "Logged out successfully" },
        {
          status: 200,
          headers: pluginResponse.headers,
        },
      );
    }
  } catch (error: unknown) {
    if (axios.isAxiosError(error) && error.response) {
      return NextResponse.json(
        { error: error.response.data || "Backend Error" },
        { status: error.response.status },
      );
    }
    return NextResponse.json({ error: "Internal Error" }, { status: 500 });
  }

  // Default handler for request
  return nativeHandlers.POST(request);
}

export async function GET(request: NextRequest) {
  return nativeHandlers.GET(request);
}

const handleAuthResponse = async (
  response: AxiosAuthResponse,
): Promise<NextResponse> => {
  const pluginResponse = await auth.api.customAuth({
    body: response.data,
    asResponse: true,
  });

  const responseData = await pluginResponse.json();

  return NextResponse.json(responseData, {
    status: pluginResponse.status,
    headers: pluginResponse.headers,
  });
};
