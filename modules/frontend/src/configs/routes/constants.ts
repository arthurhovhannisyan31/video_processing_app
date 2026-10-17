/** biome-ignore-all lint/style/useLiteralEnumMembers: not relevant */
export enum RootKeys {
  SignIn = "signin",
  Signup = "signup",
  ForgotPassword = "forgot-password",
  ResetPassword = "reset-password",
  Video = "video",
}

export enum RootPath {
  SignIn = RootKeys.SignIn,
  Signup = RootKeys.Signup,
  ForgotPassword = RootKeys.ForgotPassword,
  ResetPassword = RootKeys.ResetPassword,
  Video = RootKeys.Video,
}

/** Pages available without an authenticated session */
export const PUBLIC_PATHS: RootPath[] = [
  RootPath.SignIn,
  RootPath.Signup,
  RootPath.ForgotPassword,
  RootPath.ResetPassword,
];
