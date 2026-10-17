import zod from "zod";

export const forgotPasswordSchema = zod.object({
  email: zod.string().email("Please enter a valid email address"),
});

export type ForgotPasswordSchema = zod.output<typeof forgotPasswordSchema>;
