import type {
  Socket,
  SocketDelegate,
  SocketPolicy,
} from "@github/stable-socket";
import type { VideoStateProgress } from "generated/client";

import {
  WS_RECONNECT_ATTEMPTS,
  WS_RECONNECT_TIMEOUT_TIME,
} from "components/modules/video/constants";
import { store } from "store";
import { videoStore } from "store/video";

export const websocketPolicy: SocketPolicy = {
  timeout: WS_RECONNECT_TIMEOUT_TIME,
  attempts: WS_RECONNECT_ATTEMPTS,
};
let retryCount = WS_RECONNECT_ATTEMPTS;

export const wsDelegateConfig: SocketDelegate = {
  socketDidOpen: (_) => {},
  socketDidReceiveMessage: (_socket: Socket, message: string) => {
    let stateProgress: VideoStateProgress;

    try {
      stateProgress = JSON.parse(message) as VideoStateProgress;

      store.set(videoStore, {
        progress: Math.round(stateProgress.value * 100),
        done: stateProgress.done,
      });
    } catch (err) {
      console.warn(err);

      return;
    }
  },
  socketDidClose: (_socket: Socket, _code?: number, _reason?: string) => {},
  socketShouldRetry: (_socket: Socket, _code: number): boolean =>
    --retryCount > 0,
  socketDidFinish: (_socket: Socket) => {},
};
