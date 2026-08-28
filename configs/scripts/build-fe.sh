#!/bin/bash

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "${REPO_ROOT}/modules/frontend" || exit

yarn generate-openapi
yarn next build