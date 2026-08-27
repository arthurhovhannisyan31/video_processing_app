import type { AxiosError } from "axios";

export type ApiError = AxiosError & {
  error: string;
};
