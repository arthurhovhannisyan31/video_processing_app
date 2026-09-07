"use client";

// This is temporary solution for user session_id generation
import { store } from "store";
import { userStore } from "store/user";

export const getUserId = () => {
  const userState = store.get(userStore);

  return userState.id;
};
