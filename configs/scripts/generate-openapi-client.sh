#!/bin/bash

REPO_ROOT="$(git rev-parse --show-toplevel)"

yarn openapi-ts --client=@hey-api/client-axios \
  -i "https://api.videoprocessing.app/api/openapi" \
  -o "${REPO_ROOT}/modules/frontend/src/generated/client"