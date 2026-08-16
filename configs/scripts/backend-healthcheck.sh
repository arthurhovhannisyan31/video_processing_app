#!/bin/bash
curl -kf http://${BACKEND_HOST}:${BACKEND_HTTP_PORT}/api/health || exit 1