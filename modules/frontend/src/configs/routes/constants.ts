/** biome-ignore-all lint/style/useLiteralEnumMembers: not relevant */
export enum RootKeys {
  Home = "",
  SignIn = "signin",
  Signup = "signup",
  ForgotPassword = "forgot-password",
  ResetPassword = "reset-password",
}

export enum RootPath {
  Home = RootKeys.Home,
  SignIn = RootKeys.SignIn,
  Signup = RootKeys.Signup,
  ForgotPassword = RootKeys.ForgotPassword,
  ResetPassword = RootKeys.ResetPassword,
}

/** Pages available without an authenticated session */
export const PUBLIC_PATHS: RootPath[] = [
  RootPath.Home,
  RootPath.SignIn,
  RootPath.Signup,
  RootPath.ForgotPassword,
  RootPath.ResetPassword,
];
