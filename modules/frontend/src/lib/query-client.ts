import { QueryClient, type QueryClientConfig } from "@tanstack/react-query";
import { HttpStatusCode } from "axios";

const MAX_RETRIES = 1;
const HTTP_STATUS_TO_NOT_RETRY = [
  HttpStatusCode.BadRequest,
  HttpStatusCode.Unauthorized,
  HttpStatusCode.Forbidden,
  HttpStatusCode.NotFound,
];

export interface ApiError extends Error {
  status?: number;
}

export const config: QueryClientConfig = {
  defaultOptions: {
    queries: {
      staleTime: 60 * 60 * 1000,
      gcTime: 5 * 60 * 60 * 1000,
      refetchOnWindowFocus: false,
      retry: (failureCount, error: ApiError) => {
        if (
          failureCount > MAX_RETRIES ||
          HTTP_STATUS_TO_NOT_RETRY.includes(error?.status ?? 0)
        ) {
          return false;
        }

        return true;
      },
    },
  },
};

export const queryClient = new QueryClient(config);
