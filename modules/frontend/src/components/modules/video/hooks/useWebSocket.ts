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
import { getUserId } from "lib/helpers/getUserId";

export type WebSocketResult = BufferedWebSocketResult;

export const useWebSocket = (): WebSocketResult => {
  const retryCountRef = useRef(WS_RECONNECT_ATTEMPTS);
  const wsConnectURI = useMemo(() => {
    const userId = getUserId();

    return `${API_WS_URL}/video/ws/${userId}`;
  }, []);
  const wsDelegateConfig = useMemo(
    () => getWsDelegateConfig(retryCountRef),
    [],
  );

  return useBufferedWebSocket(
    wsConnectURI,
    true,
    wsDelegateConfig,
    websocketPolicy,
  );
};
