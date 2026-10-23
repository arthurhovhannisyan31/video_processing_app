import { atom, type PrimitiveAtom } from "jotai";

export interface UserStore {
  resetToken: PrimitiveAtom<string>;
}

export const userStore: UserStore = {
  resetToken: atom(""),
};
