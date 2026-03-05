use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use sqlx::SqlitePool;
use tower::ServiceExt;

use ethnomusicology_backend::api::claude::{
    CacheMetrics, ClaudeClientTrait, ClaudeError, RequestContentBlock,
};
use ethnomusicology_backend::routes::setlist::{setlist_router, SetlistRouteState};

// ---------------------------------------------------------------------------
// MockClaude: returns generation JSON from generate_with_blocks,
// verification JSON from generate_setlist (different methods → no counter needed)
// ---------------------------------------------------------------------------

struct VerifyMockClaude {
    generation_response: String,
    verification_response: String,
}

#[async_trait::async_trait]
impl ClaudeClientTrait for VerifyMockClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        // Called by verify_setlist() second pass
        Ok(self.verification_response.clone())
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        // Called by generate_setlist_from_request() initial generation
        Ok((self.generation_response.clone(), CacheMetrics::default()))
    }
}

/// Generation succeeds, but verification returns an error.
struct VerifyErrorMockClaude {
    generation_response: String,
}

#[async_trait::async_trait]
impl ClaudeClientTrait for VerifyErrorMockClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Err(ClaudeError::Api(
            "Verification service unavailable".to_string(),
        ))
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Ok((self.generation_response.clone(), CacheMetrics::default()))
    }
}

/// Simple mock: always returns the same response for both methods.
struct SimpleMockClaude {
    response: String,
}

#[async_trait::async_trait]
impl ClaudeClientTrait for SimpleMockClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Ok(self.response.clone())
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Ok((self.response.clone(), CacheMetrics::default()))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn create_test_pool() -> SqlitePool {
    ethnomusicology_backend::db::create_test_pool().await
}

fn build_app(pool: SqlitePool, claude: Arc<dyn ClaudeClientTrait>) -> axum::Router {
    let state = Arc::new(SetlistRouteState { pool, claude });
    setlist_router(state)
}

async fn post_json(
    app: axum::Router,
    uri: &str,
    body: serde_json::Value,
) -> (u16, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .header("X-User-Id", "dev-user")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    (status, json)
}

async fn get_json(app: axum::Router, uri: &str) -> (u16, serde_json::Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    (status, json)
}

fn generation_json() -> String {
    serde_json::json!({
        "tracks": [
            {
                "position": 1,
                "title": "Desert Rose",
                "artist": "Sting",
                "bpm": 102.0,
                "key": "A minor",
                "camelot": "8A",
                "energy": 4,
                "transition_note": "Open with atmospheric pads",
                "confidence": "high",
                "source": "suggestion"
            },
            {
                "position": 2,
                "title": "Fake Track Title",
                "artist": "Nonexistent Artist",
                "bpm": 128.0,
                "key": "B minor",
                "camelot": "9A",
                "energy": 7,
                "transition_note": "Build energy",
                "confidence": "medium",
                "source": "suggestion"
            }
        ],
        "notes": "Test setlist"
    })
    .to_string()
}

fn verification_json_with_flag() -> String {
    serde_json::json!({
        "tracks": [
            {
                "position": 1,
                "title": "Desert Rose",
                "artist": "Sting",
                "confidence": "high",
                "flag": null,
                "correction": null
            },
            {
                "position": 2,
                "title": "Fake Track Title",
                "artist": "Nonexistent Artist",
                "confidence": "low",
                "flag": "no_such_track",
                "correction": "This track does not exist in the artist's discography"
            }
        ],
        "summary": "One track flagged as non-existent"
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// 1. verify=true happy path: confidence adjusted and flags present in response
#[tokio::test]
async fn test_verify_true_propagates_flags_in_response() {
    let pool = create_test_pool().await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(VerifyMockClaude {
        generation_response: generation_json(),
        verification_response: verification_json_with_flag(),
    });

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test", "verify": true }),
    )
    .await;

    assert_eq!(status, 201);
    let tracks = json["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);

    // Track 1: no flag, confidence stays high
    assert_eq!(tracks[0]["confidence"], "high");
    assert!(tracks[0]["verification_flag"].is_null());
    assert!(tracks[0]["verification_note"].is_null());

    // Track 2: flagged as no_such_track, confidence downgraded to low
    assert_eq!(tracks[1]["confidence"], "low");
    assert_eq!(tracks[1]["verification_flag"], "no_such_track");
    assert_eq!(
        tracks[1]["verification_note"],
        "This track does not exist in the artist's discography"
    );
}

/// 2. verify=false/omitted: tracks returned with original confidence, no flags
#[tokio::test]
async fn test_verify_false_no_flags() {
    let pool = create_test_pool().await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(SimpleMockClaude {
        response: generation_json(),
    });

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test", "verify": false }),
    )
    .await;

    assert_eq!(status, 201);
    let tracks = json["tracks"].as_array().unwrap();

    // No verification run: original confidence preserved, no flags
    assert_eq!(tracks[0]["confidence"], "high");
    assert_eq!(tracks[1]["confidence"], "medium");
    assert!(tracks[0]["verification_flag"].is_null());
    assert!(tracks[1]["verification_flag"].is_null());
}

/// 2b. verify omitted entirely: same as verify=false
#[tokio::test]
async fn test_verify_omitted_no_flags() {
    let pool = create_test_pool().await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(SimpleMockClaude {
        response: generation_json(),
    });

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;

    assert_eq!(status, 201);
    let tracks = json["tracks"].as_array().unwrap();
    assert_eq!(tracks[0]["confidence"], "high");
    assert!(tracks[0]["verification_flag"].is_null());
}

/// 3. verify=true but verification call fails: graceful degradation, note added
#[tokio::test]
async fn test_verify_true_verification_error_graceful_degradation() {
    let pool = create_test_pool().await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(VerifyErrorMockClaude {
        generation_response: generation_json(),
    });

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test", "verify": true }),
    )
    .await;

    // Still returns 201 — verification failure is non-fatal
    assert_eq!(status, 201);
    let tracks = json["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);

    // Original confidence preserved (no verification applied)
    assert_eq!(tracks[0]["confidence"], "high");
    assert_eq!(tracks[1]["confidence"], "medium");

    // No flags set
    assert!(tracks[0]["verification_flag"].is_null());
    assert!(tracks[1]["verification_flag"].is_null());

    // Notes should mention verification unavailable
    let notes = json["notes"].as_str().unwrap_or("");
    assert!(
        notes.contains("Verification unavailable"),
        "Expected 'Verification unavailable' in notes, got: {notes}"
    );
}

/// 4. DB round-trip: confidence and flags persisted and returned from GET
#[tokio::test]
async fn test_verify_db_round_trip() {
    let pool = create_test_pool().await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(VerifyMockClaude {
        generation_response: generation_json(),
        verification_response: verification_json_with_flag(),
    });

    // Generate with verify=true
    let (status, gen_json) = post_json(
        build_app(pool.clone(), claude.clone()),
        "/setlists/generate",
        serde_json::json!({ "prompt": "db round trip test", "verify": true }),
    )
    .await;

    assert_eq!(status, 201);
    let setlist_id = gen_json["id"].as_str().unwrap();

    // GET the setlist back and verify DB-persisted confidence and flags
    let (get_status, get_json) =
        get_json(build_app(pool, claude), &format!("/setlists/{setlist_id}")).await;

    assert_eq!(get_status, 200);
    let tracks = get_json["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);

    // Track 1: high confidence, no flag persisted
    assert_eq!(tracks[0]["confidence"], "high");
    assert!(tracks[0]["verification_flag"].is_null());

    // Track 2: low confidence and flag persisted
    assert_eq!(tracks[1]["confidence"], "low");
    assert_eq!(tracks[1]["verification_flag"], "no_such_track");
    assert_eq!(
        tracks[1]["verification_note"],
        "This track does not exist in the artist's discography"
    );
}
