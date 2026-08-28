#!/bin/bash
set -e # Exit immediately if a command exits with a non-zero status

REPO_ROOT="$(git rev-parse --show-toplevel)"
OUTPUT_DIR="${REPO_ROOT}/modules/frontend/src/generated/client"

# Default schema source (Remote)
SCHEMA_SRC="https://api.videoprocessing.app/api/openapi"

# Check for the local flag
if [[ "$1" == "-l" || "$1" == "--local" ]]; then
    SCHEMA_SRC="${REPO_ROOT}/modules/backend/openapi.json"
    echo "Using LOCAL schema: ${SCHEMA_SRC}"
else
    echo "Using REMOTE schema: ${SCHEMA_SRC}"
fi

# Run the openapi-ts generator
yarn openapi-ts --client=@hey-api/client-axios \
  -i "$SCHEMA_SRC" \
  -o "$OUTPUT_DIR"