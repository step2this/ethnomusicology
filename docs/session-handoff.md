# Session Handoff

## Current State (2026-03-17)

- **Branch**: `main`
- **Status**: Serverless migration COMPLETE — Lambda + Vercel + Neon live at `tarab.studio`

## What Was Done This Session

### Serverless Migration (SP-011 + ST-012 + ST-013)

1. **SP-011 Lambda Spike**: Confirmed Axum + lambda_http compiles (8.3MB binary). Fixed OpenSSL → rustls-only.
2. **ST-012 Lambda-Ready Backend** (4 parallel builders + critic):
   - T1/T5: Dual-mode main.rs, removed static file serving, TOKEN_ENCRYPTION_KEY required in Lambda
   - T2: CSRF HashMap → signed JWT (HS256) for Spotify OAuth state
   - T3: Removed `tokio::spawn` background enrichment
   - T4: `Instant` → `chrono::Utc` for token caching (audio.rs, soundcloud.rs)
   - T6: Config changes (dotenvy skip, frontend_url, pool size 2/5, migration skip, AtomicBool removal)
   - T7: Two-pass critic review, findings addressed
3. **ST-013 Deploy**:
   - Lambda deployed: `ethnomusicology-api` (us-east-1, 300s timeout, 256MB, Function URL)
   - Vercel deployed: `frontend-next` project, all routes rendering
   - DNS: `tarab.studio` A record → 76.76.21.21 (Vercel)
   - CI/CD: deploy.yml → cargo-lambda, e2e.yml → DATABASE_URL secret
   - GitHub secrets: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, DATABASE_URL

### Hotfixes During Deploy
- Restored dev-user seed in Lambda mode (FK constraint on OAuth callback)
- Added `default-user` seed (frontend uses `default-user`, not `dev-user`)
- Changed all handler defaults from `dev-user` to `default-user`
- Added Spotify import error detail logging

## Architecture (NEW)

- **Frontend**: Next.js 16 on Vercel (rewrites `/api/*` to Lambda Function URL)
- **Backend**: Rust/Axum on AWS Lambda (Function URL, 300s timeout, 256MB)
- **Database**: Neon Postgres (pooler endpoint)
- **Auth**: Hardcoded `default-user` / `X-User-Id` header (Clerk is next phase)
- **DNS**: Route53 A record → Vercel (76.76.21.21)
- **CI/CD**: GitHub Actions → cargo-lambda deploy (backend), Vercel auto-deploy (frontend)
- **Lambda Function URL**: `https://w7crmq4hdlg4ae7fdqk7pk2lgu0eyvew.lambda-url.us-east-1.on.aws/`
- **IAM Role**: `ethnomusicology-lambda-exec`

## Test Counts

- Backend: 379 (down from 381 — removed 2 tests for deleted features)
- Frontend vitest: 102
- Playwright e2e: 38
- Total: 519

## Known Issues

- **Spotify import 403**: Token may be expired by time user tries to import. No auto-refresh implemented. Re-authorize to get fresh token. Pre-existing issue, not caused by migration.
- **EC2 still running**: Has not been terminated yet. DNS points to Vercel but EC2 instance still exists (costs ~$70/mo).

## Next Steps

1. **Investigate Spotify 403**: May need token refresh flow or check if scopes are sufficient
2. **EC2 teardown**: Terminate instance, release Elastic IP ($3.65/mo saved)
3. **Clerk auth**: Separate plan — replace X-User-Id with real auth
4. **Known debt**: Update `.claude/rules/known-debt.md` with deferred critic findings
5. **Vercel GitHub integration**: Connect repo for auto-deploy on push (currently manual via CLI)
