#!/bin/bash

DOMAIN="api.videoprocessing.app"

# Expose the .well-known/acme-challenge/ directory so Let's Encrypt can verify the domain.

cp nginx.certbot.conf nginx.conf

# Start services including certbot and nginx
docker compose --file docker-compose.prod.yml up -d nginx

# Trigger a request to Let's Encrypt using the Certbot container via the webroot method
docker compose --file docker-compose.prod.yml run --rm certbot certonly --webroot --webroot-path=/var/www/certbot --no-eff-email --non-interactive --agree-tos --register-unsafely-without-email -d "${DOMAIN}"

# Restore production nginx configuration to use generated ssl certificates from linked volumes
cp nginx.prod.conf nginx.conf

# Recreate nginx service to serve 443 port
docker compose --file docker-compose.prod.yml up -d --force-recreate nginx

echo "Server configuration is done"