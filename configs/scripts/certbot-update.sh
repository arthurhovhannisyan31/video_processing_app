#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Target log file
LOG_FILE="/var/log/certbot-renew.log"

# Run the renewal process in a redirected block
{
    # Move to the project directory or exit if it fails
    cd /root/server || exit 1

    echo
    echo "============================"
    date

    # Run the Certbot renewal container
    docker compose -f docker-compose.prod.yml run --rm -T certbot renew --quiet

    # Reload Nginx configuration to pick up the new certificates
    docker compose -f docker-compose.prod.yml exec -T nginx nginx -s reload

} >> "$LOG_FILE" 2>&1