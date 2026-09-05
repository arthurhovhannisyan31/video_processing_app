import { useMemo } from "react";

import {
  websocketPolicy,
  wsDelegateConfig,
} from "components/modules/video/ws-configs";
import { API_WS_URL } from "configs/constants";
import { useBufferedWebSocket } from "hooks/useBufferedWebSocket";
import { getUserId } from "lib/helpers/getUserId";

export const useWebSocket = (): void => {
  const wsConnectURI = useMemo(() => {
    const userId = getUserId();

    return `${API_WS_URL}/video/ws/${userId}`;
  }, []);

  useBufferedWebSocket(wsConnectURI, true, wsDelegateConfig, websocketPolicy);
};
