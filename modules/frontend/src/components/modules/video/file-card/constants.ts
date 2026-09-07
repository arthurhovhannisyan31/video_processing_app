import type { AttachmentStateType } from "components/ui/attachment";

import { Status } from "components/modules/video/constants";

export const statusToAttachmentStateMap: Record<Status, AttachmentStateType> = {
  [Status.Stale]: "idle",
  [Status.Pending]: "processing",
  [Status.Done]: "done",
  [Status.Error]: "error",
};
