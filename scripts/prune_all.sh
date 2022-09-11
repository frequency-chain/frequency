#!/bin/sh

# Print each command and exit on error
set -e

echo "[INFO] Running system prune..."
docker system prune -a --volumes
echo "[INFO] Removing unused containers..."
docker container prune -f
echo "[INFO] Removing unused images..."
docker image prune -a -f

echo "[INFO] After cleanup:"
docker container ls
docker image ls
