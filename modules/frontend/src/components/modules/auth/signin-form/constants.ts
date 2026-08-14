import zod from "zod";

export const signInSchema = zod.object({
  email: zod.string().email("Please enter a valid email address"),
  password: zod.string().min(8, "Password must be at least 8 characters long"),
});

export type SignInSchema = zod.output<typeof signInSchema>;
