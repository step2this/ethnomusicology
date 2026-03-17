# SP-011: Lambda Feasibility Spike

## Hypothesis
Axum 0.8 + sqlx (Postgres) + lambda_http compile to a binary small enough for Lambda and can serve HTTP requests via Lambda Function URL.

## Time-box
30 minutes

## Findings

### Compilation: PASS
- `lambda_http` v1.1.1 integrates cleanly with Axum 0.8 via Tower Service trait
- Dual-mode `main()` works: detects Lambda via `AWS_LAMBDA_RUNTIME_API` env var
- `#[tokio::main]` is compatible — lambda_http uses the existing tokio runtime

### OpenSSL Issue: RESOLVED
- `cargo lambda build` cross-compiles to Linux and requires no native OpenSSL
- `rspotify` and `reqwest` pulled in `native-tls` via default features
- Fix: `default-features = false` on both `reqwest` and `rspotify`, use `rustls-tls` only
- Local build and Lambda build both work after this change

### Binary Size: EXCELLENT
- Release binary: **8.3 MB** (uncompressed)
- Compressed: **~3.6 MB** (gzip estimate)
- Lambda limits: 50 MB zipped (direct upload), 250 MB unzipped
- Headroom: 13x under the direct upload limit

### Build Times
- Local debug build: ~43s (incremental after first build)
- Lambda release build: ~4m 22s (cross-compilation with strip + LTO)

### Rust Toolchain
- Rust 1.93.0 (well above lambda_http MSRV of 1.84.0)
- cargo-lambda 1.9.1 with Zig 0.16.0-dev for cross-compilation

### Key Code Pattern
```rust
if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
    lambda_http::run(app).await
} else {
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await
}
```

## Remaining Postconditions (deferred to deployment)
- PC3: Lambda serves health endpoint — requires AWS account setup
- PC4: Local dev unchanged — verified (`cargo build` + `cargo clippy` pass)

## Decision
**PROCEED** to ST-012 (Lambda-Ready Backend). Compilation and binary size are not blockers. The OpenSSL/rustls fix was the only issue encountered.

## Dependencies Changed
- `reqwest`: added `default-features = false` (rustls-only, no OpenSSL)
- `rspotify`: added `default-features = false` (rustls-only, no OpenSSL)
- `lambda_http = "1.1"`: new dependency
- `[profile.release]`: `strip = true`, `lto = true`
