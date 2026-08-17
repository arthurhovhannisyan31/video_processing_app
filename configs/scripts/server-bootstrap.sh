#!/bin/bash
# Run once on the Hetzner host, as root, before the first server-deploy.yml run.
# Idempotent — safe to re-run.
#
# Prerequisite: api.videoprocessing.app (A record) must already point to this
# host's IP, otherwise certbot below will fail to validate the domain.
set -euo pipefail

DOMAIN="api.videoprocessing.app"

if ! command -v nginx >/dev/null 2>&1; then
  apt update
  apt install -y nginx certbot python3-certbot-nginx
fi

if command -v ufw >/dev/null 2>&1; then
  ufw allow 80/tcp
  ufw allow 443/tcp
fi

if [ ! -d "/etc/letsencrypt/live/${DOMAIN}" ]; then
  systemctl stop nginx 2>/dev/null || true
  certbot certonly --standalone -d "${DOMAIN}" --non-interactive --agree-tos --register-unsafely-without-email
fi

systemctl enable nginx
systemctl start nginx

echo "Bootstrap done. Push to main (or run 'gh workflow run server-deploy.yml') to deploy."
