import {
  passwordSchema,
  passwordsMatch,
  passwordsMismatchError,
} from "components/modules/auth/common/constants";
import zod from "zod";

export const TOKEN_QUERY_PARAM = "token";

export const resetPasswordSchema = zod
  .object({
    password: passwordSchema,
    confirmPassword: zod.string(),
  })
  .refine(passwordsMatch, passwordsMismatchError);

export type ResetPasswordSchema = zod.output<typeof resetPasswordSchema>;
