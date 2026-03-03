# AWS Deployment Plan: Simple Production Setup

## Current State (Already Working!)

We're further along than expected. The app is **already deployed and serving**:

- **EC2**: t3.large, us-east-1f, Ubuntu 24.04
- **Caddy**: Installed, running, auto-HTTPS via Let's Encrypt
- **Domain**: `salamic-vibes.duckdns.org` (DuckDNS free dynamic DNS)
- **Auth**: Basic auth on all routes except Spotify callback
- **Frontend**: Served from `frontend/build/web` via Caddy file_server
- **Backend**: Reverse proxied on `/api/*` to `localhost:3001`
- **AWS CLI**: Configured with `sst-deployer` IAM user
- **SST artifacts**: S3 buckets suggest prior SST usage
- **No Docker, no Terraform, no Route53 zones**

## What's Missing (Gap Analysis)

| Gap | Risk | Effort |
|-----|------|--------|
| No systemd service for backend | Backend dies on crash, no auto-restart | Low |
| No CI/CD pipeline | Manual SSH deploy, error-prone | Medium |
| No SQLite backups | Data loss on disk failure | Low |
| DuckDNS domain | Unprofessional, no DNS control | Low |
| No Elastic IP | Instance restart changes public IP | Low |
| No monitoring | Won't know if app is down | Low |
| Secrets in unknown state | API keys may be in plaintext | Low |
| 76% disk usage (36G/48G) | Build artifacts filling up | Low |

## Recommended Approach: Harden What Exists

**Philosophy**: Don't migrate. Don't containerize. Don't add IaC. Just harden the existing setup with 6 targeted improvements.

### Phase 1: Foundation (Must-Have)

**T1: Systemd service for backend**
- Create `/etc/systemd/system/ethnomusicology.service`
- Environment variables for secrets (CLAUDE_API_KEY, SPOTIFY_CLIENT_ID, etc.)
- `Restart=on-failure` with 10s backoff
- `journalctl -u ethnomusicology` for logs

**T2: Elastic IP**
- Allocate EIP, associate with current instance
- Ensures IP survives instance stop/start

**T3: SQLite backup to S3**
- Cron job: every 6 hours, `sqlite3 .backup` → `aws s3 cp` to dedicated bucket
- Retain 30 days of backups
- Cost: ~$0.10/month

### Phase 2: CI/CD

**T4: GitHub Actions deploy pipeline**
- On push to `main` (after PR merge):
  1. Run quality gates (already done by E2E workflow)
  2. Build release binary: `cargo build --release`
  3. Build Flutter web: `flutter build web`
  4. SCP binary + web assets to EC2
  5. `systemctl restart ethnomusicology`
- Requires: EC2 SSH key as GitHub secret, EC2 host as secret

**T5: Real domain (optional but recommended)**
- Register domain or use existing one
- Route53 hosted zone → A record pointing to Elastic IP
- Caddy auto-provisions new Let's Encrypt cert for new domain
- Cost: ~$12/year for domain + $0.50/month Route53

### Phase 3: Observability

**T6: Basic health monitoring**
- Simple approach: cron job that curls `/api/health` every 5 min
- If fail → send SNS notification (email/SMS)
- OR: Use free-tier UptimeRobot (external monitoring, zero AWS cost)

**T7: Log rotation + disk cleanup**
- Configure Caddy log rotation (already logging to file)
- Clean up Rust build artifacts: `cargo clean` on old targets
- Add logrotate config for app logs

## What We're NOT Doing (And Why)

| Temptation | Why Skip |
|------------|----------|
| Docker/containers | Single binary deploys via SCP. Containers add build time + registry overhead for zero benefit. |
| App Runner / ECS Fargate | Over-engineered for one app on one server. Adds $20-40/month for managed compute we don't need. |
| RDS PostgreSQL | SQLite handles < 1000 users fine. Adds $32+/month. Migrate when we actually need it. |
| CloudFront + S3 for frontend | Caddy already serves static files. CDN adds complexity. Do this when we have global users. |
| Terraform / CDK / CloudFormation | One EC2 instance doesn't need IaC. Shell scripts are fine. IaC when we have >3 resources to manage. |
| AWS Secrets Manager | $0.40/secret/month. Systemd environment overrides are free and sufficient. |
| CloudWatch Logs Agent | Free UptimeRobot + journalctl is enough for MVP. CloudWatch when we need dashboards. |
| Multi-AZ / Load Balancer | Single point of failure is acceptable for MVP. ALB adds $22/month. |

## MCP Plugin Recommendation

Based on research, three options exist:

| Plugin | What It Does | Our Verdict |
|--------|-------------|-------------|
| **AWS CLI MCP** | Execute AWS CLI via natural language | **Skip** — we already have `aws` CLI configured and working |
| **AWS IaC MCP** | CloudFormation/CDK generation + validation | **Skip** — no IaC needed for 1 server |
| **Terraform MCP** | Terraform registry + workspace management | **Skip** — no Terraform needed |

**Recommendation: No plugins needed.** Our deployment is simple enough that raw `aws` CLI commands (which we already have) cover everything. MCP plugins would encourage us to build infrastructure we don't need.

If we later scale to multi-service architecture, the **AWS IaC MCP Server** (official AWS Labs) would be the right choice for CloudFormation template generation.

## Cost Estimate

| Item | Current | After Plan |
|------|---------|------------|
| EC2 t3.large | ~$60/mo | Same (consider downsizing to t3.medium ~$30/mo) |
| EBS 48GB | ~$4.80/mo | Same |
| Elastic IP | $0 | $3.65/mo (free when attached to running instance, $3.65 if stopped) |
| S3 backups | $0 | ~$0.10/mo |
| Route53 (optional) | $0 | ~$0.50/mo |
| Domain (optional) | $0 | ~$1/mo ($12/year) |
| **Total** | **~$65/mo** | **~$65-70/mo** |

**Potential savings**: Downsize from t3.large to t3.medium (4GB RAM → still plenty for Rust + SQLite). Saves ~$30/mo.

## Timeline

- T1-T3 (Foundation): ~2 hours
- T4 (CI/CD): ~3 hours
- T5 (Domain): ~1 hour
- T6-T7 (Observability): ~1 hour
- **Total: ~7 hours**

## Success Criteria

1. Backend auto-restarts on crash (systemd)
2. `git push` to main triggers automatic deploy
3. SQLite backed up to S3 every 6 hours
4. Team gets notified if app goes down
5. Disk usage under control
