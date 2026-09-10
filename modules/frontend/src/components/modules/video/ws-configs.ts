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
import { debounce } from "lodash-es";
import { store } from "store";
import { videoStore } from "store/video";

export const websocketPolicy: SocketPolicy = {
  timeout: WS_RECONNECT_TIMEOUT_TIME,
  attempts: WS_RECONNECT_ATTEMPTS,
};
let retryCount = WS_RECONNECT_ATTEMPTS;

const debouncedUpdaters = new Map<string, ReturnType<typeof debounce>>();
const getDebouncedUpdater = (fileName: string) => {
  if (!debouncedUpdaters.has(fileName)) {
    debouncedUpdaters.set(
      fileName,
      debounce(
        (stateProgress: VideoStateProgress) => {
          const videoState = store.get(videoStore);
          store.set(videoStore, {
            ...videoState,
            [stateProgress.file_name]: {
              progress: Math.round(stateProgress.value * 100),
              done: stateProgress.done,
            },
          });
        },
        200,
        { trailing: true, leading: true },
      ),
    );
  }
  return debouncedUpdaters.get(fileName);
};

export const wsDelegateConfig: SocketDelegate = {
  socketDidOpen: (_) => {},
  socketDidReceiveMessage: (_socket: Socket, message: string) => {
    try {
      const stateProgress: VideoStateProgress = JSON.parse(message);
      const updateFn = getDebouncedUpdater(stateProgress.file_name);

      if (!updateFn) return;

      updateFn(stateProgress);
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
