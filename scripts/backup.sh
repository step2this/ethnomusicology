#!/bin/bash
set -euo pipefail

DB_PATH="/opt/ethnomusicology/data/ethnomusicology.db"
BACKUP_PATH="/tmp/ethnomusicology-backup-$(date +%Y%m%d-%H%M%S).db"
BUCKET="ethnomusicology-backups"

# Create clean backup using VACUUM INTO (compacted copy, safe during writes)
sqlite3 "$DB_PATH" "VACUUM INTO '$BACKUP_PATH'"

# Verify integrity
RESULT=$(sqlite3 "$BACKUP_PATH" "PRAGMA integrity_check")
if [ "$RESULT" != "ok" ]; then
  echo "BACKUP INTEGRITY CHECK FAILED: $RESULT" >&2
  rm -f "$BACKUP_PATH"
  exit 1
fi

# Upload to S3
aws s3 cp "$BACKUP_PATH" "s3://$BUCKET/$(date +%Y/%m/%d)/$(basename "$BACKUP_PATH")"
rm -f "$BACKUP_PATH"

# Prune backups older than 30 days (uses S3 ls date column, not filename parsing)
aws s3 ls "s3://$BUCKET/" --recursive \
  | awk -v cutoff="$(date -d '30 days ago' +%Y-%m-%d)" '$1 < cutoff {print $4}' \
  | xargs -r -I{} aws s3 rm "s3://$BUCKET/{}"

echo "$(date): Backup complete → s3://$BUCKET/"
