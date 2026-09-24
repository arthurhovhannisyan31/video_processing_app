import { useMemo } from "react";

import {
  websocketPolicy,
  wsDelegateConfig,
} from "components/modules/video/ws-configs";
import { API_WS_URL } from "configs/constants";
import {
  type BufferedWebSocketResult,
  useBufferedWebSocket,
} from "hooks/useBufferedWebSocket";
import { getUserId } from "lib/helpers/getUserId";

export type WebSocketResult = BufferedWebSocketResult;

export const useWebSocket = (): WebSocketResult => {
  const wsConnectURI = useMemo(() => {
    const userId = getUserId();

    return `${API_WS_URL}/video/ws/${userId}`;
  }, []);

  return useBufferedWebSocket(
    wsConnectURI,
    true,
    wsDelegateConfig,
    websocketPolicy,
  );
};
