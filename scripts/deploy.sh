#!/usr/bin/env bash
set -euo pipefail
IMAGE="${1:?Usage: ./scripts/deploy.sh <image-tag>}"
WEB_IMAGE="${IMAGE/\/myrix-chain:/\/myrix-chain-web:}"
export MYRIX_IMAGE="$IMAGE"
export MYRIX_WEB_IMAGE="$WEB_IMAGE"
docker compose -f docker-compose.deploy.yml pull
docker compose -f docker-compose.deploy.yml up -d --remove-orphans
docker compose -f docker-compose.deploy.yml ps
for i in {1..30}; do
  if curl -fsS http://127.0.0.1:3000/health >/dev/null; then
    echo "MYRIX API healthy: $IMAGE"
    exit 0
  fi
  sleep 2
done
docker compose -f docker-compose.deploy.yml logs --tail=100 myrix-api
exit 1
