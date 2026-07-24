#!/bin/bash

REPO_ROOT="$(git rev-parse --show-toplevel)"

. "${REPO_ROOT}/configs/bash/colors"

printf "⚠️ Run cargo fmt check\n\n"

cargo fmt -- --check

latest_exit_status=$?

if [[ ${latest_exit_status} -ne 0 ]]; then
  printf "⛔ ${RED} There are some code style issues, run 'cargo fmt' first \n\n${NC}"
  exit 1
fi

exit 0