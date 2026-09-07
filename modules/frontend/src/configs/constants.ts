export const IS_PROD = process.env.NEXT_PUBLIC_IS_PROD === "true";
export const API_DOMAIN = process.env.NEXT_PUBLIC_API_DOMAIN ?? "";
const API_HTTP_PROTOCOL = IS_PROD ? "https" : "http";
export const API_HTTP_URL = `${API_HTTP_PROTOCOL}://${API_DOMAIN}`;
const API_WS_PROTOCOL = IS_PROD ? "wss" : "ws";
export const API_WS_URL = `${API_WS_PROTOCOL}://${API_DOMAIN}`;

export const BETTER_AUTH_SECRET = process.env.BETTER_AUTH_SECRET;
export const CALLBACK_URL = "callback-url";

export const SESSION_DATA_KEY = IS_PROD
  ? "__Secure-better-auth.session_data"
  : "better-auth.session_data";
export const MAX_BODY_SIZE = process.env.NEXT_PUBLIC_MAX_BODY_SIZE;
export const PROXY_AUTH_CHECK_ENABLED =
  process.env.NEXT_PUBLIC_PROXY_AUTH_CHECK_ENABLED === "true";
export const X_USER_ID_HEADER = "X-USER-ID";
