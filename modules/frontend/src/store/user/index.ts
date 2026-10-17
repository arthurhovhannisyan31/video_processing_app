import type { PrimitiveAtom } from "jotai/vanilla/atom";

import { atom } from "jotai";

export interface UserStore {
  resetToken: PrimitiveAtom<string>;
}

export const userStore: UserStore = {
  resetToken: atom(""),
};
