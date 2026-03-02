#!/usr/bin/env sh
set -eu

TARGET_REF="${1:-HEAD~1}"

echo "Rolling back to ${TARGET_REF}..."
git checkout "${TARGET_REF}"

sh deploy/deploy.sh
