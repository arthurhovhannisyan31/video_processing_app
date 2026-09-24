import { type RefObject, useCallback, useEffect, useRef } from "react";

import {
  BufferedSocket,
  type SocketDelegate,
  type SocketPolicy,
  StableSocket,
} from "@github/stable-socket";

export interface BufferedWebSocketResult {
  ref: RefObject<BufferedSocket | null>;
  openConnection: () => Promise<void>;
}

export const useBufferedWebSocket = (
  url: string,
  isReady: boolean,
  delegate: SocketDelegate,
  policy: SocketPolicy,
): BufferedWebSocketResult => {
  const ref = useRef<BufferedSocket | null>(null);

  const openConnection = useCallback(async () => {
    if (isReady && ref.current && !ref.current?.isOpen()) {
      await ref.current.open();
    }
  }, [isReady]);

  useEffect(() => {
    if (isReady && ref.current === null) {
      ref.current = new BufferedSocket(new StableSocket(url, delegate, policy));

      void ref.current.open();
    }

    return () => {
      if (ref.current?.isOpen()) {
        ref.current.close();
      }
    };
  }, [delegate, isReady, policy, url]);

  return {
    ref,
    openConnection,
  };
};
