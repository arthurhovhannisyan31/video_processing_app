import { HttpStatusCode } from "axios";
import {
  CALLBACK_URL,
  PROXY_AUTH_CHECK_ENABLED,
  SESSION_DATA_KEY,
} from "configs/constants";
import { RootPath } from "configs/routes/constants";
import { getTokenExpired } from "lib/helpers/auth";
import { type NextRequest, NextResponse } from "next/server";

export default async function proxy(
  request: NextRequest,
): Promise<NextResponse | undefined> {
  if (!PROXY_AUTH_CHECK_ENABLED) {
    return NextResponse.next();
  }

  const { origin, searchParams } = request.nextUrl;

  const fullPath =
    `${request.nextUrl.pathname}${request.nextUrl.search}`.replace("/", "");
  const rootPath = (fullPath.split("?")[0].split("/")[0] ?? "") as RootPath;

  const callbackUrl = searchParams.get(CALLBACK_URL);

  if ([RootPath.SignIn, RootPath.Signup].includes(rootPath)) {
    return undefined;
  }

  /* Session validation */
  const sessionToken = request.cookies.get(SESSION_DATA_KEY);
  const isTokenExpired = sessionToken?.value
    ? await getTokenExpired(sessionToken?.value)
    : true;

  /* Unauthenticated user or expired token access validation */
  if (!sessionToken || isTokenExpired) {
    const callback = callbackUrl || encodeURIComponent(request.url);

    return NextResponse.redirect(
      `${origin}/${RootPath.SignIn}?${CALLBACK_URL}=${callback}`,
      {
        status: HttpStatusCode.SeeOther,
      },
    );
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    {
      source:
        "/((?!api|_vercel|_next/static|_next/image|_next/data|favicon.ico|robots.txt|.well-known*).*)",
    },
  ],
  unstable_allowDynamic: ["**/node_modules/lodash-es/**/*.js"],
};
