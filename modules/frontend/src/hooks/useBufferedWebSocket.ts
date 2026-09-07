import { type RefObject, useEffect, useRef } from "react";

import {
  BufferedSocket,
  type SocketDelegate,
  type SocketPolicy,
  StableSocket,
} from "@github/stable-socket";

export const useBufferedWebSocket = (
  url: string,
  isReady: boolean,
  delegate: SocketDelegate,
  policy: SocketPolicy,
): RefObject<BufferedSocket | null> => {
  const wsRef = useRef<BufferedSocket | null>(null);

  useEffect(() => {
    if (isReady && wsRef.current === null) {
      wsRef.current = new BufferedSocket(
        new StableSocket(url, delegate, policy),
      );

      void wsRef.current.open();
    }

    return () => {
      if (wsRef.current?.isOpen()) {
        wsRef.current.close();
      }
    };
  }, [delegate, isReady, policy, url]);

  return wsRef;
};
