import type { FilesStateMap } from "components/modules/video/types";
import { type FC, useMemo } from "react";

import FileCard from "components/modules/video/file-card";

export interface FilesListProps {
  files: File[];
  filesStateMap: FilesStateMap;
}

export const FilesList: FC<FilesListProps> = ({ files, filesStateMap }) => {
  const cards = useMemo(
    () =>
      files.map((file) => {
        const state = filesStateMap[file.name];

        return (
          <FileCard
            key={file.name}
            file={file}
            progress={state.progress}
            inspectData={state.inspectionData}
            status={state.status}
            progressType={state.progressType}
            jobType={state.jobType}
            error={state.error}
            processedData={state.processedData}
          />
        );
      }),
    [files, filesStateMap],
  );

  return (
    <div className={"flex flex-col gap-8 w-full"}>
      <div className={"flex flex-col gap-4"}>{cards}</div>
    </div>
  );
};
