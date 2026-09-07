#!/bin/bash
curl -kf http://${BACKEND_HOST}:${BACKEND_HTTP_PORT}/api/ready || exit 1