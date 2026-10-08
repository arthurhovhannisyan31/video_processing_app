import type { ApiError } from "configs/types";

export const stopImmediatePropagation = (e: Event): void => {
  e.preventDefault();
  e.stopPropagation();
};

export const getErrorMessage = (err: ApiError, defaultMessage: string) => {
  switch (err.status) {
    case 429:
      return "Too many requests. Please try again later";
    default:
      return (err.message || err.status || defaultMessage).toString();
  }
};
