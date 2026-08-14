import zod from "zod";

export const USERNAME_MIN_LENGTH = 8;
export const USERNAME_MAX_LENGTH = 255;
export const PASSWORD_MIN_LENGTH = 8;
export const PASSWORD_MAX_LENGTH = 32;

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
    password: zod
      .string()
      .min(
        PASSWORD_MIN_LENGTH,
        `Password must be at least ${PASSWORD_MIN_LENGTH} characters long`,
      )
      .max(
        PASSWORD_MAX_LENGTH,
        `Password must be at most ${PASSWORD_MAX_LENGTH} characters long`,
      ),
    confirmPassword: zod.string(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: "Passwords do not match",
    path: ["confirmPassword"],
  });

export type SignupSchema = zod.output<typeof signupSchema>;

export interface PassportStrengthValidation {
  hasUppercase: boolean;
  hasLowercase: boolean;
  hasDigits: boolean;
  hasSpecial: boolean;
  allValid: boolean;
}

export const getPasswordStrength = (
  password: string,
): PassportStrengthValidation => {
  const hasDigits = /\d/g.test(password);
  const hasLowercase = /[a-z]/g.test(password);
  const hasUppercase = /[A-Z]/g.test(password);
  const hasSpecial = /[*.!@#$%^&(){}[\]:;<>,?~_+\-=|\\/]/g.test(password);
  const allValid = hasDigits && hasLowercase && hasUppercase && hasSpecial;

  return {
    hasDigits,
    hasLowercase,
    hasUppercase,
    hasSpecial,
    allValid,
  };
};
