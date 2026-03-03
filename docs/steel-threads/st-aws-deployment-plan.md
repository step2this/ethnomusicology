# AWS Deployment Plan: Simple Production Setup

## Current State (Already Working!)

The app is **already deployed and serving**:

- **EC2**: t3.large (i-032e18c24b66bb182), us-east-1f, Ubuntu 24.04
- **Elastic IP**: 52.72.57.136 (eipalloc-018738bbc164a1ce0) — already allocated and associated
- **Caddy**: Installed, running, auto-HTTPS via Let's Encrypt
- **Domain**: `salamic-vibes.duckdns.org` (DuckDNS free dynamic DNS)
- **Auth**: Basic auth on all routes except `/api/auth/spotify/callback`
- **Frontend**: Served from `frontend/build/web` via Caddy file_server
- **Backend**: Reverse proxied on `/api/*` to `localhost:3001`
- **AWS CLI**: Configured with `sst-deployer` IAM user (**has AdministratorAccess — must scope down**)
- **Architecture**: x86_64 — CI/CD runner must match this
- **No Docker, no Terraform, no Route53 zones**

## What's Missing (Gap Analysis)

| Gap | Risk | Effort |
|-----|------|--------|
| No systemd service for backend | Backend dies on crash, no auto-restart | Low |
| `DEV_MODE` may be set; `CorsLayer::permissive()` unconditional | Dev seed endpoint exposed; any-origin CORS | Low |
| IAM user has AdministratorAccess | Full account takeover if credentials leak | Medium |
| No CI/CD pipeline | Manual SSH deploy, error-prone | Medium |
| No rollback strategy | Bad deploy = extended downtime | Medium |
| No SQLite backups | Data loss on disk failure | Low |
| No backup integrity verification | Could silently upload corrupt backups | Low |
| DuckDNS domain | Unprofessional, no DNS control | Low |
| No monitoring | Won't know if app is down | Low |
| Health endpoint doesn't check DB | App "healthy" but DB could be unreachable | Low |
| 70% disk usage (34G/48G) | Build artifacts accumulating | Low |

## Recommended Approach: Harden What Exists

**Philosophy**: Don't migrate. Don't containerize. Don't add IaC. Just harden the existing setup with targeted improvements.

---

### Phase 1: Foundation (Must-Have)

**T1: Systemd service + security hardening**

Create `/etc/systemd/system/ethnomusicology.service`:
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

Create `/etc/ethnomusicology/env` (chmod 600):
```
DEV_MODE=false
CLAUDE_API_KEY=<key>
SPOTIFY_CLIENT_ID=<id>
SPOTIFY_CLIENT_SECRET=<secret>
DATABASE_URL=sqlite:/opt/ethnomusicology/data/ethnomusicology.db?mode=rwc
BIND_ADDRESS=127.0.0.1
```

Key decisions:
- Binary lives at `/opt/ethnomusicology/ethnomusicology-backend-current` (symlink — see T4 rollback)
- Data directory: `/opt/ethnomusicology/data/` (separate from binary for clean deploys)
- `DEV_MODE=false` explicitly set — disables dev seed endpoint
- `BIND_ADDRESS=127.0.0.1` — only Caddy can reach the backend
- Secrets in systemd env file (chmod 600), NOT in Caddyfile or git

Also in T1, fix `CorsLayer::permissive()` in main.rs — replace with explicit allowed origins for the DuckDNS domain (or `localhost` for dev). This is cheap to do now and avoids a landmine when basic auth is eventually removed.

Also wire up graceful shutdown: add `tokio::signal::ctrl_c()` with `.with_graceful_shutdown()` on `axum::serve`. Without it, `systemctl restart` sends SIGTERM and active connections (e.g., mid-Spotify OAuth flow) are dropped immediately.

**T2: Scope down IAM**

The `sst-deployer` IAM user currently has `AdministratorAccess`. This must be scoped down before CI/CD secrets are stored in GitHub.

Create a new IAM policy `ethnomusicology-deploy`:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:PutObject", "s3:GetObject", "s3:ListBucket"],
      "Resource": ["arn:aws:s3:::ethnomusicology-backups", "arn:aws:s3:::ethnomusicology-backups/*"]
    }
  ]
}
```

Either: attach this to `sst-deployer` (removing AdministratorAccess) or create a new user `ethnomusicology-backup` with just this policy. CI/CD SSH access uses a separate deploy key (not AWS credentials).

**T3: SQLite backup to S3 with integrity verification**

Create `/opt/ethnomusicology/scripts/backup.sh`:
```bash
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
  # TODO: Add SNS notification here when T6 monitoring is set up
  rm -f "$BACKUP_PATH"
  exit 1
fi

# Upload to S3
aws s3 cp "$BACKUP_PATH" "s3://$BUCKET/$(date +%Y/%m/%d)/$(basename $BACKUP_PATH)"
rm -f "$BACKUP_PATH"

# Prune backups older than 30 days (uses S3 ls date column, not filename parsing)
aws s3 ls "s3://$BUCKET/" --recursive \
  | awk -v cutoff="$(date -d '30 days ago' +%Y-%m-%d)" '$1 < cutoff {print $4}' \
  | xargs -r -I{} aws s3 rm "s3://$BUCKET/{}"

echo "Backup complete: $BACKUP_PATH → s3://$BUCKET/"
```

Cron (every 6 hours): `0 */6 * * * /opt/ethnomusicology/scripts/backup.sh >> /var/log/ethnomusicology-backup.log 2>&1`

---

### Phase 2: CI/CD (with rollback)

**T4: GitHub Actions deploy pipeline with symlink rollback**

Design contract (shared between T1 and T4):
- Binary location: `/opt/ethnomusicology/ethnomusicology-backend-<timestamp>`
- Active binary symlink: `/opt/ethnomusicology/ethnomusicology-backend-current`
- Frontend location: `/opt/ethnomusicology/frontend-<timestamp>/`
- Active frontend symlink: `/opt/ethnomusicology/frontend-current`
- Service name: `ethnomusicology.service`
- Environment file: `/etc/ethnomusicology/env`

Caddy must be updated to serve from the frontend symlink:
```
root * /opt/ethnomusicology/frontend-current
```

Deploy script (`scripts/deploy.sh` on EC2):
```bash
#!/bin/bash
set -euo pipefail

DEPLOY_DIR="/opt/ethnomusicology"
TIMESTAMP=$(date +%s)
NEW_BINARY="$DEPLOY_DIR/ethnomusicology-backend-$TIMESTAMP"
CURRENT_BINARY_LINK="$DEPLOY_DIR/ethnomusicology-backend-current"
CURRENT_FRONTEND_LINK="$DEPLOY_DIR/frontend-current"
NEW_FRONTEND="$DEPLOY_DIR/frontend-$TIMESTAMP"
PREVIOUS_BINARY=$(readlink -f "$CURRENT_BINARY_LINK" 2>/dev/null || echo "")
PREVIOUS_FRONTEND=$(readlink -f "$CURRENT_FRONTEND_LINK" 2>/dev/null || echo "")

# Binary was already SCP'd to $NEW_BINARY by GitHub Actions
chmod +x "$NEW_BINARY"

# Frontend was already SCP'd to $NEW_FRONTEND/ by GitHub Actions
# Swap both symlinks atomically — no partial state where Caddy serves
# old index.html with new chunk hashes (or vice versa)
ln -sf "$NEW_BINARY" "${CURRENT_BINARY_LINK}.tmp"
mv -f "${CURRENT_BINARY_LINK}.tmp" "$CURRENT_BINARY_LINK"

ln -sfn "$NEW_FRONTEND" "${CURRENT_FRONTEND_LINK}.tmp"
mv -f "${CURRENT_FRONTEND_LINK}.tmp" "$CURRENT_FRONTEND_LINK"

# Restart service (Caddy picks up new frontend via symlink, no restart needed)
sudo systemctl restart ethnomusicology

# Health check (60 second timeout with exponential backoff)
# Cold-start Axum + SQLite migrations can take time on a loaded instance
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
  mv -f "${CURRENT_BINARY_LINK}.tmp" "$CURRENT_BINARY_LINK"
fi
if [ -n "$PREVIOUS_FRONTEND" ] && [ -d "$PREVIOUS_FRONTEND" ]; then
  ln -sfn "$PREVIOUS_FRONTEND" "${CURRENT_FRONTEND_LINK}.tmp"
  mv -f "${CURRENT_FRONTEND_LINK}.tmp" "$CURRENT_FRONTEND_LINK"
fi
sudo systemctl restart ethnomusicology
echo "Rollback complete. Previous binary + frontend restored."
exit 1
```

GitHub Actions workflow (`.github/workflows/deploy.yml`):
```yaml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    needs: []  # Runs independently; e2e.yml is a separate check
    steps:
      - uses: actions/checkout@v5

      - uses: dtolnay/rust-toolchain@stable
      - uses: subosito/flutter-action@v2
        with: { channel: stable }

      - name: Build backend (release)
        run: cd backend && cargo build --release

      - name: Build frontend (web)
        run: cd frontend && flutter build web --release

      - name: Deploy to EC2
        env:
          EC2_SSH_KEY: ${{ secrets.EC2_SSH_KEY }}
          EC2_HOST: ${{ secrets.EC2_HOST }}
        run: |
          mkdir -p ~/.ssh
          echo "$EC2_SSH_KEY" > ~/.ssh/deploy_key && chmod 600 ~/.ssh/deploy_key
          ssh-keyscan "$EC2_HOST" >> ~/.ssh/known_hosts 2>/dev/null
          TIMESTAMP=$(date +%s)
          SSH="ssh -i ~/.ssh/deploy_key ubuntu@$EC2_HOST"
          SCP="scp -i ~/.ssh/deploy_key"
          # Upload binary
          $SCP backend/target/release/ethnomusicology-backend \
            ubuntu@$EC2_HOST:/opt/ethnomusicology/ethnomusicology-backend-$TIMESTAMP
          # Upload frontend to timestamped directory (atomic swap, not file-by-file)
          $SSH "mkdir -p /opt/ethnomusicology/frontend-$TIMESTAMP"
          $SCP -r frontend/build/web/* \
            ubuntu@$EC2_HOST:/opt/ethnomusicology/frontend-$TIMESTAMP/
          # Run deploy script (handles symlink swap, restart, health check, rollback)
          $SSH "TIMESTAMP=$TIMESTAMP /opt/ethnomusicology/scripts/deploy.sh"
```

Note: The SSH deploy key is a **dedicated key pair** for deployment only. It is NOT the `sst-deployer` AWS credentials. The EC2 instance's AWS CLI access is scoped to S3 backups only (T2).

Architecture assumption: GitHub Actions `ubuntu-latest` is x86_64, matching our EC2 t3.large. If we ever switch to Graviton (ARM), the workflow needs `runs-on: ubuntu-latest-arm64`.

**T5: Real domain (optional but recommended)**
- Register domain or use existing one
- Route53 hosted zone → A record pointing to Elastic IP (52.72.57.136)
- Update Caddyfile with new domain (manual SSH operation — Caddy is NOT managed by CI/CD)
- Caddy auto-provisions new Let's Encrypt cert
- Cost: ~$12/year for domain + $0.50/month Route53

---

### Phase 3: Observability

**T6: Health monitoring with DB check**

Add `/api/health/ready` endpoint to backend that also runs `SELECT 1` against SQLite:
```rust
async fn health_ready(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => Json(json!({"status": "ok", "db": "ok"})),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"status": "error", "db": e.to_string()}))),
    }
}
```

Monitoring options (pick one):
- **Free**: UptimeRobot (external, zero AWS cost, email/SMS alerts)
- **AWS**: Cron + curl + SNS: `curl -sf https://salamic-vibes.duckdns.org/api/health/ready || aws sns publish ...`

**T7: Disk cleanup + log rotation**

Immediate:
- `git worktree prune` (done — recovered 2.6 GB)
- `cd backend && cargo clean` on stale build targets
- Remove any orphaned `.claude/worktrees/` directories

Ongoing:
- Logrotate config for `/var/log/caddy/*.log` and `/var/log/ethnomusicology-backup.log`
- Deploy script keeps only last 3 binaries (built into T4 deploy.sh)
- Consider expanding EBS from 48G to 100G ($5/mo difference) if disk pressure returns

---

## Caddy Configuration (Manual, NOT in CI/CD)

The Caddyfile at `/etc/caddy/Caddyfile` is managed manually via SSH. The CI/CD pipeline does NOT touch it. Current config includes:
- Basic auth (bcrypt hash) on all routes except Spotify callback
- Reverse proxy `/api/*` → `localhost:3001`
- File server for Flutter web assets
- Access logging to `/var/log/caddy/`

When T5 (real domain) is done, the Caddyfile domain line must be updated manually.

---

## What We're NOT Doing (And Why)

| Temptation | Why Skip | Trigger to Reconsider |
|------------|----------|----------------------|
| Docker/containers | Single binary deploys via SCP | When deploy needs >1 server, or build time >10 min |
| App Runner / ECS Fargate | Over-engineered for 1 app on 1 server | When we need auto-scaling or zero-ops compute |
| RDS PostgreSQL | SQLite handles < 1000 users fine. +$32/mo | When concurrent writers exceed SQLite single-writer limit, or user count > 500 |
| CloudFront + S3 for frontend | Caddy already serves static files | When serving >1 GB/day static, or users in 3+ geographic regions |
| Terraform / CDK / CloudFormation | 1 EC2 + 1 S3 bucket doesn't need IaC | When managing >5 AWS resources, or when staging env is needed |
| AWS Secrets Manager | Systemd env file is sufficient. +$0.40/secret/mo | When >10 developers, rotation automation needed, or audit logging required |
| CloudWatch Logs Agent | journalctl + UptimeRobot is enough | When we need dashboards or log search across time |
| Multi-AZ / Load Balancer | SPOF acceptable for MVP. ALB +$22/mo | When uptime SLA >99% or business depends on availability |
| Migration versioning | Current `CREATE IF NOT EXISTS` is safe | **When the first `ALTER TABLE` migration is needed** — must add version tracking before deploying it |

## MCP Plugin Recommendation

**No plugins needed.** Raw `aws` CLI covers everything for 1 server. MCP plugins would encourage infrastructure we don't need.

Revisit when moving to multi-service architecture: **AWS IaC MCP Server** (official AWS Labs) for CloudFormation.

## Cost Estimate

| Item | Current | After Plan |
|------|---------|------------|
| EC2 t3.large | ~$60/mo | Consider downsizing to t3.medium (~$30/mo) |
| EBS 48GB | ~$4.80/mo | Same |
| Elastic IP | ~$3.65/mo | Same (already allocated) |
| S3 backups | $0 | ~$0.10/mo |
| Route53 (optional) | $0 | ~$0.50/mo |
| Domain (optional) | $0 | ~$1/mo ($12/year) |
| **Total** | **~$69/mo** | **~$69-74/mo** (or ~$39/mo if downsized) |

## Timeline

- T1-T3 (Foundation + IAM + backups): ~3 hours
- T4 (CI/CD with rollback): ~3 hours
- T5 (Domain): ~1 hour
- T6-T7 (Observability + cleanup): ~1 hour
- **Total: ~8 hours**

## Success Criteria

1. Backend runs as systemd service, auto-restarts on crash
2. `DEV_MODE=false` in production, secrets in env file (not plaintext)
3. IAM scoped to S3 backup permissions only
4. `git push` to main triggers automatic deploy with health-check rollback
5. SQLite backed up to S3 every 6 hours with integrity verification
6. Team gets notified if app goes down (UptimeRobot or SNS)
7. Disk usage under control with log rotation and artifact cleanup

## Known Debt (Added)

| Item | Priority | Notes |
|------|----------|-------|
| Migration versioning | Medium | `CREATE IF NOT EXISTS` works now; need version tracking before first `ALTER TABLE` |
| CI/CD architecture assumption (x86_64) | Low | Document in workflow; update if switching to Graviton |

## Review Notes (Claude Web, 2026-03-03)

Incorporated feedback from external review:
1. Health check window extended to 60s with exponential backoff (was 30s linear)
2. Frontend deploy now uses atomic symlink swap (was file-by-file SCP mid-serve)
3. Backup pruning uses S3 `ls` date column (was fragile filename parsing)
4. `CorsLayer::permissive()` moved from debt to T1 implementation (fix while we're there)
5. `actions/checkout@v5` confirmed working (our e2e.yml already uses it)
6. Added graceful shutdown (`with_graceful_shutdown`) to T1 — drain in-flight requests on SIGTERM
