import { atom } from "jotai";

export interface ProcessingProgress {
  progress: number;
  done: boolean;
}

export type VideoStore = Record<string, ProcessingProgress>;

export const videoStore = atom<VideoStore>({});
