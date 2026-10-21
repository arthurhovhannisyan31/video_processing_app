import {
  type DragEvent,
  type Ref,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { stopImmediatePropagation } from "helpers/utils";

const events: (keyof GlobalEventHandlersEventMap)[] = [
  "drop",
  "dragover",
  "dragenter",
];

export interface DnDEventResult {
  isOver: boolean;
  ref?: Ref<HTMLDivElement>;
}

export const useHoverEvent = (
  cb?: (e: DragEvent<HTMLInputElement>) => void,
): DnDEventResult => {
  const [isOver, setIsOver] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const updateOverState = useCallback((e: DragEvent<HTMLInputElement>) => {
    switch (e.type) {
      case "dragenter":
      case "dragover": {
        const isChildrenEvent = !!ref.current?.contains(e.target as Node);
        setIsOver(isChildrenEvent);

        return;
      }
      case "drop": {
        setIsOver(false);
      }
    }
  }, []);

  const eventHandler = useCallback(
    (e: unknown) => {
      stopImmediatePropagation(e as Event);
      updateOverState(e as DragEvent<HTMLInputElement>);

      cb?.(e as DragEvent<HTMLInputElement>);
    },
    [cb, updateOverState],
  );

  useEffect(() => {
    events.forEach((event) => {
      document.addEventListener(event, eventHandler);
    });

    return () => {
      events.forEach((event) => {
        document.removeEventListener(event, eventHandler);
      });
    };
  }, [eventHandler]);

  return {
    isOver,
    ref,
  };
};
