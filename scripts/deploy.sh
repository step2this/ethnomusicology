#!/bin/bash
set -euo pipefail

DEPLOY_DIR="/opt/ethnomusicology"
TIMESTAMP="${TIMESTAMP:-$(date +%s)}"
NEW_BINARY="$DEPLOY_DIR/ethnomusicology-backend-$TIMESTAMP"
CURRENT_BINARY_LINK="$DEPLOY_DIR/ethnomusicology-backend-current"
CURRENT_FRONTEND_LINK="$DEPLOY_DIR/frontend-current"
NEW_FRONTEND="$DEPLOY_DIR/frontend-$TIMESTAMP"
PREVIOUS_BINARY=$(readlink -f "$CURRENT_BINARY_LINK" 2>/dev/null || echo "")
PREVIOUS_FRONTEND=$(readlink -f "$CURRENT_FRONTEND_LINK" 2>/dev/null || echo "")

# Binary was already SCP'd to $NEW_BINARY by GitHub Actions
chmod +x "$NEW_BINARY"

# Swap both symlinks atomically
# NOTE: must use mv -Tf (no-target-directory) because mv -f follows
# the existing symlink if it points to a directory, moving the .tmp
# file INTO the directory instead of replacing the symlink.
ln -sf "$NEW_BINARY" "${CURRENT_BINARY_LINK}.tmp"
mv -Tf "${CURRENT_BINARY_LINK}.tmp" "$CURRENT_BINARY_LINK"

if [ -d "$NEW_FRONTEND" ]; then
  ln -sfn "$NEW_FRONTEND" "${CURRENT_FRONTEND_LINK}.tmp"
  mv -Tf "${CURRENT_FRONTEND_LINK}.tmp" "$CURRENT_FRONTEND_LINK"
fi

# Restart service (Caddy picks up new frontend via symlink, no restart needed)
sudo systemctl restart ethnomusicology

# Health check (60 second timeout with exponential backoff)
DELAY=1
for i in $(seq 1 10); do
  if curl -sf http://localhost:3001/api/health > /dev/null 2>&1; then
    echo "Deploy successful: $NEW_BINARY"
    # Keep last 3 binaries + frontends, remove older ones
    ls -t "$DEPLOY_DIR"/ethnomusicology-backend-[0-9]* 2>/dev/null | tail -n +4 | xargs -r rm -f
    ls -dt "$DEPLOY_DIR"/frontend-[0-9]* 2>/dev/null | tail -n +4 | xargs -r rm -rf
    exit 0
  fi
  sleep "$DELAY"
  DELAY=$((DELAY * 2 > 15 ? 15 : DELAY * 2))
done

# Health check failed — rollback both binary and frontend
echo "HEALTH CHECK FAILED — rolling back" >&2
if [ -n "$PREVIOUS_BINARY" ] && [ -f "$PREVIOUS_BINARY" ]; then
  ln -sf "$PREVIOUS_BINARY" "${CURRENT_BINARY_LINK}.tmp"
  mv -Tf "${CURRENT_BINARY_LINK}.tmp" "$CURRENT_BINARY_LINK"
fi
if [ -n "$PREVIOUS_FRONTEND" ] && [ -d "$PREVIOUS_FRONTEND" ]; then
  ln -sfn "$PREVIOUS_FRONTEND" "${CURRENT_FRONTEND_LINK}.tmp"
  mv -Tf "${CURRENT_FRONTEND_LINK}.tmp" "$CURRENT_FRONTEND_LINK"
fi
sudo systemctl restart ethnomusicology
echo "Rollback complete. Previous binary + frontend restored."
exit 1
