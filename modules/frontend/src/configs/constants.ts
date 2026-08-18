export const NEXT_PUBLIC_BACKEND_BASE_URL =
  process.env.NEXT_PUBLIC_BACKEND_BASE_URL;
export const BETTER_AUTH_SECRET = process.env.BETTER_AUTH_SECRET;
export const CALLBACK_URL = "callback-url";
export const IS_PROD = process.env.NEXT_PUBLIC_IS_PROD === "true";
export const SESSION_DATA_KEY = IS_PROD
  ? "__Secure-better-auth.session_data"
  : "better-auth.session_data";
