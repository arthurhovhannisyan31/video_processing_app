import { type RefObject, useRef } from "react";

export interface AbortControllerResult {
  ref: RefObject<AbortController | null>;
  init: () => void;
  abort: () => void;
}

export const useAbortController = (): AbortControllerResult => {
  const abortControllerRef = useRef<AbortController | null>(null);
  const initController = () => {
    abortControllerRef.current = new AbortController();
  };
  const abort = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
  };

  return {
    ref: abortControllerRef,
    init: initController,
    abort,
  };
};
