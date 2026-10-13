import { useMemo, useRef } from "react";

import { WS_RECONNECT_ATTEMPTS } from "components/modules/video/constants";
import {
  getWsDelegateConfig,
  websocketPolicy,
} from "components/modules/video/ws-configs";
import { API_WS_URL } from "configs/constants";
import {
  type BufferedWebSocketResult,
  useBufferedWebSocket,
} from "hooks/useBufferedWebSocket";

export type WebSocketResult = BufferedWebSocketResult;

const WS_CONNECT_URI = `${API_WS_URL}/video/ws`;

export const useWebSocket = (): WebSocketResult => {
  const retryCountRef = useRef(WS_RECONNECT_ATTEMPTS);
  const wsDelegateConfig = useMemo(
    () => getWsDelegateConfig(retryCountRef),
    [],
  );

  return useBufferedWebSocket(
    WS_CONNECT_URI,
    wsDelegateConfig,
    websocketPolicy,
  );
};
