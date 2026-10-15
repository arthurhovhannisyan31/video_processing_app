import {
  passwordSchema,
  passwordsMatch,
  passwordsMismatchError,
} from "components/modules/auth/common/constants";
import zod from "zod";

export const USERNAME_MIN_LENGTH = 8;
export const USERNAME_MAX_LENGTH = 255;

export const signupSchema = zod
  .object({
    username: zod
      .string()
      .min(
        USERNAME_MIN_LENGTH,
        `Username must be at least ${USERNAME_MIN_LENGTH} characters long`,
      )
      .max(
        USERNAME_MAX_LENGTH,
        `Username must be at most ${USERNAME_MAX_LENGTH} characters long`,
      ),
    email: zod.string().email("Please enter a valid email address"),
    password: passwordSchema,
    confirmPassword: zod.string(),
  })
  .refine(passwordsMatch, passwordsMismatchError);

export type SignupSchema = zod.output<typeof signupSchema>;
