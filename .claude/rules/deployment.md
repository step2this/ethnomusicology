---
paths:
  - "scripts/**"
  - ".github/workflows/**"
  - "backend/src/main.rs"
---

# Deployment — tarab.studio

## Infrastructure

- **Host**: EC2 instance (Ubuntu), elastic IP → Route53 → `tarab.studio`
- **TLS / reverse proxy**: Caddy (HTTPS termination, serves Flutter web from symlink)
- **API**: Axum binary running as a systemd service on port 3001 (bound to 127.0.0.1)
- **Database**: SQLite at `/opt/ethnomusicology/data/ethnomusicology.db`
- **Migrations**: `sqlx::migrate!()` applied on startup — never run manual SQL migrations

## Directory Layout (on EC2)

```
/opt/ethnomusicology/
  ethnomusicology-backend-<timestamp>   # release binaries (keep last 3)
  ethnomusicology-backend-current       # symlink → current binary
  frontend-<timestamp>/                 # Flutter web builds (keep last 3)
  frontend-current -> frontend-<ts>/    # symlink → current frontend
  data/
    ethnomusicology.db                  # SQLite database
  scripts/
    deploy.sh                           # atomic swap + health check + rollback
    backup.sh                           # SQLite → S3 backup
    ethnomusicology.service             # systemd unit file
/etc/ethnomusicology/env                # secret env vars (chmod 600)
```

## systemd Service

Unit file: `/opt/ethnomusicology/scripts/ethnomusicology.service`
(symlinked or copied to `/etc/systemd/system/ethnomusicology.service`)

```ini
[Unit]
Description=Ethnomusicology DJ API
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/opt/ethnomusicology
ExecStart=/opt/ethnomusicology/ethnomusicology-backend-current
Restart=on-failure
RestartSec=10s
StandardOutput=journal
StandardError=journal
EnvironmentFile=/etc/ethnomusicology/env

[Install]
WantedBy=multi-user.target
```

Key commands:
```bash
sudo systemctl restart ethnomusicology
sudo systemctl status ethnomusicology
sudo journalctl -u ethnomusicology -f
```

## Environment Config

File: `/etc/ethnomusicology/env` (chmod 600, not committed to git)
Template: `scripts/env.template`

Required variables:
```
DEV_MODE=false
BIND_ADDRESS=127.0.0.1
PORT=3001
DATABASE_URL=sqlite:/opt/ethnomusicology/data/ethnomusicology.db?mode=rwc
SPOTIFY_CLIENT_ID=
SPOTIFY_CLIENT_SECRET=
SPOTIFY_REDIRECT_URI=https://tarab.studio/api/auth/spotify/callback
ANTHROPIC_API_KEY=
TOKEN_ENCRYPTION_KEY=          # openssl rand -base64 32
```

## Deploy Process (`scripts/deploy.sh`)

GitHub Actions SCP's the binary and frontend to timestamped paths, then calls this script with `TIMESTAMP` set.

1. `chmod +x` the new binary
2. Atomic symlink swap (via `ln -sf` + `mv -f .tmp`) for both binary and frontend
3. `systemctl restart ethnomusicology`
4. Health check: `GET http://localhost:3001/api/health` with exponential backoff (up to 60s, 10 attempts)
5. On success: prune old binaries/frontends (keep last 3)
6. On failure: restore previous binary and frontend symlinks, restart, exit 1

Caddy serves the Flutter web via the `frontend-current` symlink — no Caddy restart needed on frontend updates.

## CI/CD (`.github/workflows/deploy.yml`)

Triggers on push to `main`.

Steps:
1. Checkout + install Rust toolchain (stable) + Flutter (stable, cached)
2. Cache Rust build artifacts (`backend/target`, `~/.cargo`)
3. `cargo build --release` (working-directory: `backend/`)
4. `flutter clean && flutter pub get && flutter build web --release` (working-directory: `frontend/`)
5. SCP binary to `/opt/ethnomusicology/ethnomusicology-backend-<timestamp>`
6. SCP frontend build to `/opt/ethnomusicology/frontend-<timestamp>/`
7. SSH: `TIMESTAMP=$TIMESTAMP /opt/ethnomusicology/scripts/deploy.sh`

Required GitHub Secrets: `EC2_SSH_KEY`, `EC2_HOST`

## Database Backup (`scripts/backup.sh`)

Runs via cron every 6 hours.

- Uses `sqlite3 VACUUM INTO` for a clean, compacted backup (safe during live writes)
- Runs `PRAGMA integrity_check` to verify backup before upload
- Uploads to S3: `s3://ethnomusicology-backups/<YYYY/MM/DD>/ethnomusicology-backup-<timestamp>.db`
- Prunes S3 objects older than 30 days

Cron entry (ubuntu user):
```
0 */6 * * * /opt/ethnomusicology/scripts/backup.sh >> /var/log/ethnomusicology-backup.log 2>&1
```

## DNS

Route53 hosted zone → A record for `tarab.studio` → EC2 elastic IP
