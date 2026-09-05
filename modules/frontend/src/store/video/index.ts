import { atom } from "jotai";

export interface VideoStore {
  progress: number;
  done: boolean;
}

export const videoStore = atom<VideoStore>({
  progress: 0,
  done: false,
});
