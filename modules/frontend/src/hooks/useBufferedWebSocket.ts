import { type RefObject, useCallback, useEffect, useRef } from "react";

import {
  BufferedSocket,
  type SocketDelegate,
  type SocketPolicy,
  StableSocket,
} from "@github/stable-socket";

export interface BufferedWebSocketResult {
  ref: RefObject<BufferedSocket | null>;
  wsReconnect: () => Promise<void>;
}

export const useBufferedWebSocket = (
  url: string,
  delegate: SocketDelegate,
  policy: SocketPolicy,
): BufferedWebSocketResult => {
  const ref = useRef<BufferedSocket | null>(null);

  const wsReconnect = useCallback(async () => {
    if (ref.current && !ref.current?.isOpen()) {
      await ref.current.open();
    }
  }, []);

  useEffect(() => {
    if (ref.current === null) {
      ref.current = new BufferedSocket(new StableSocket(url, delegate, policy));
    }

    return () => {
      if (ref.current?.isOpen()) {
        ref.current.close();
        ref.current = null;
      }
    };
  }, [delegate, policy, url]);

  return {
    ref,
    wsReconnect,
  };
};
