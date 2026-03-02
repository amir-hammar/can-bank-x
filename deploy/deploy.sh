#!/usr/bin/env sh
set -eu

echo "Stopping previous stack (if any)..."
docker compose down --remove-orphans || true

echo "Building and starting stack..."
docker compose up -d --build

echo "Running services:"
docker compose ps
