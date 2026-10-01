import type {
  Socket,
  SocketDelegate,
  SocketPolicy,
} from "@github/stable-socket";
import type { VideoStateProgress } from "generated/client";
import type { RefObject } from "react";

import {
  WS_RECONNECT_ATTEMPTS,
  WS_RECONNECT_TIMEOUT_TIME,
} from "components/modules/video/constants";
import { WSCodes } from "configs/types";
import { debounce } from "lodash-es";
import { store } from "store";
import { videoStore } from "store/video";

export const websocketPolicy: SocketPolicy = {
  timeout: WS_RECONNECT_TIMEOUT_TIME,
  attempts: WS_RECONNECT_ATTEMPTS,
};

const RETRIABLE_WS_CODES = [
  WSCodes.GoingAway,
  WSCodes.NoStatusReceived,
  WSCodes.AbnormalClosure,
  WSCodes.InternalError,
  WSCodes.ServiceRestart,
  WSCodes.TryAgainLater,
];

const debouncedUpdatersMap = new Map<string, ReturnType<typeof debounce>>();
const getDebouncedUpdater = (fileName: string) => {
  if (!debouncedUpdatersMap.has(fileName)) {
    debouncedUpdatersMap.set(
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
  return debouncedUpdatersMap.get(fileName);
};

export const getWsDelegateConfig = (
  retryCoundRef: RefObject<number>,
): SocketDelegate => ({
  socketDidOpen: (_) => {
    // Connection is successfully opened
    retryCoundRef.current = WS_RECONNECT_ATTEMPTS;
  },
  socketDidReceiveMessage: (_socket: Socket, message: string) => {
    try {
      const stateProgress: VideoStateProgress = JSON.parse(message);
      const updateFn = getDebouncedUpdater(stateProgress.file_name);

      if (!updateFn) return;

      updateFn(stateProgress);

      if (stateProgress.done) {
        debouncedUpdatersMap.delete(stateProgress.file_name);
      }
    } catch (err) {
      console.warn(err);
      return;
    }
  },
  socketDidClose: (_socket: Socket, _code?: number, _reason?: string) => {},
  socketShouldRetry: (_socket: Socket, code: number): boolean => {
    if (!RETRIABLE_WS_CODES.includes(code)) {
      return false;
    }

    retryCoundRef.current -= 1;
    return retryCoundRef.current > 0;
  },
  socketDidFinish: (_socket: Socket) => {},
});
