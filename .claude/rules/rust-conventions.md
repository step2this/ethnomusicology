---
description: Rust/backend conventions for the ethnomusicology backend
globs: backend/**
---

# Rust Conventions

## Module Organization
- Feature modules in `src/` (e.g., `db/`, `services/`, `routes/`)
- Public API via `mod.rs` re-exports
- Traits at boundaries for testability (`ImportRepository`, `ClaudeClientTrait`, `MusicSourceClient`)

## Error Handling
- `anyhow::Result` for internal functions
- API responses: `{"error": {"code": "ERROR_CODE", "message": "Human-readable message"}}`
- Error codes: SCREAMING_SNAKE_CASE (e.g., `TURN_LIMIT_EXCEEDED`, `NOT_FOUND`, `LLM_ERROR`)

## Database Patterns
- Postgres via `sqlx` (PgPool) with `sqlx::migrate!()` for migrations
- Migrations in `backend/migrations/` numbered sequentially
- `create_test_pool()` in `db/mod.rs` for unit tests, separate `create_test_pool()` in `tests/*.rs` for integration tests — both must include ALL migrations

## Route Patterns
- Axum handlers in `routes/` modules
- State via `axum::extract::State(AppState)` with shared `Arc<dyn Trait>`
- JSON request/response with `axum::Json`
- Path params via `axum::extract::Path`

## Testing
- `cargo test` for all tests
- `cargo clippy -- -D warnings` must pass
- `cargo fmt --check` must pass
- Integration tests in `tests/` directory
