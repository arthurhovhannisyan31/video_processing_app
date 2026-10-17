import zod from "zod";

export const PASSWORD_MIN_LENGTH = 8;
export const PASSWORD_MAX_LENGTH = 100;
export const PASSWORD_RESET_COOLDOWN_SEC = 60;
// Keep in sync with backend BACKEND_PASSWORD_RESET_TTL_MIN
export const PASSWORD_RESET_TTL_MIN = 30;

export const passwordSchema = zod
  .string()
  .min(
    PASSWORD_MIN_LENGTH,
    `Password must be at least ${PASSWORD_MIN_LENGTH} characters long`,
  )
  .max(
    PASSWORD_MAX_LENGTH,
    `Password must be at most ${PASSWORD_MAX_LENGTH} characters long`,
  );

export const passwordsMatch = (data: {
  password: string;
  confirmPassword: string;
}) => data.password === data.confirmPassword;

export const passwordsMismatchError = {
  message: "Passwords do not match",
  path: ["confirmPassword"],
};

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
