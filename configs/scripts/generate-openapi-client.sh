#!/bin/bash

REPO_ROOT="$(git rev-parse --show-toplevel)"

yarn openapi-ts --client=@hey-api/client-axios \
  -i "${REPO_ROOT}/modules/backend/openapi.json" \
  -o "${REPO_ROOT}/modules/frontend/src/generated/client"