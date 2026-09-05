import { atom } from "jotai";
import { v4 as uuidv4 } from "uuid";

export interface UserStore {
  id: string;
}

export const userStore = atom<UserStore>({
  id: uuidv4(),
});
