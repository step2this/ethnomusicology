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
// Mock Claude client for integration tests
// ---------------------------------------------------------------------------

struct MockClaude {
    response: String,
}

#[async_trait::async_trait]
impl ClaudeClientTrait for MockClaude {
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

struct ErrorClaude;

#[async_trait::async_trait]
impl ClaudeClientTrait for ErrorClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Err(ClaudeError::Api("Service unavailable".to_string()))
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Err(ClaudeError::Api("Service unavailable".to_string()))
    }
}

struct RateLimitedClaude;

#[async_trait::async_trait]
impl ClaudeClientTrait for RateLimitedClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Err(ClaudeError::RateLimited {
            retry_after_secs: 10,
        })
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Err(ClaudeError::RateLimited {
            retry_after_secs: 10,
        })
    }
}

struct TimeoutClaude;

#[async_trait::async_trait]
impl ClaudeClientTrait for TimeoutClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Err(ClaudeError::Timeout)
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Err(ClaudeError::Timeout)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let migration_001 = include_str!("../migrations/001_initial_schema.sql");
    sqlx::raw_sql(migration_001).execute(&pool).await.unwrap();

    let migration_002 = include_str!("../migrations/002_spotify_imports.sql");
    sqlx::raw_sql(migration_002).execute(&pool).await.unwrap();

    let migration_003 = include_str!("../migrations/003_dj_metadata.sql");
    sqlx::raw_sql(migration_003).execute(&pool).await.unwrap();

    let migration_004 = include_str!("../migrations/004_setlists.sql");
    sqlx::raw_sql(migration_004).execute(&pool).await.unwrap();

    let migration_005 = include_str!("../migrations/005_enrichment.sql");
    sqlx::raw_sql(migration_005).execute(&pool).await.unwrap();

    let migration_006 = include_str!("../migrations/006_import_tracks.sql");
    sqlx::raw_sql(migration_006).execute(&pool).await.unwrap();

    sqlx::raw_sql("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

async fn seed_tracks(pool: &SqlitePool) {
    // Insert 3 tracks with DJ metadata
    for (id, title, bpm, key, energy) in [
        ("t1", "Desert Rose", 102.0, "8A", 4),
        ("t2", "Habibi Ya Nour El Ain", 128.0, "9A", 7),
        ("t3", "Sufi Trance", 132.0, "10A", 8),
    ] {
        sqlx::query(
            "INSERT INTO tracks (id, title, source, bpm, camelot_key, energy) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(title)
        .bind("spotify")
        .bind(bpm)
        .bind(key)
        .bind(energy as f64)
        .execute(pool)
        .await
        .unwrap();
    }

    // Artists
    sqlx::query("INSERT INTO artists (id, name) VALUES ('a1', 'Sting')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO artists (id, name) VALUES ('a2', 'Amr Diab')")
        .execute(pool)
        .await
        .unwrap();

    // Link
    sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ('t1', 'a1')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ('t2', 'a2')")
        .execute(pool)
        .await
        .unwrap();
}

fn multi_track_llm_json() -> String {
    r#"{
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
                "source": "catalog",
                "track_id": "t1"
            },
            {
                "position": 2,
                "title": "Habibi Ya Nour El Ain",
                "artist": "Amr Diab",
                "bpm": 128.0,
                "key": "B minor",
                "camelot": "9A",
                "energy": 7,
                "transition_note": "Build energy with high-pass filter",
                "source": "catalog",
                "track_id": "t2"
            },
            {
                "position": 3,
                "title": "Sufi Trance",
                "artist": "Unknown",
                "bpm": 132.0,
                "key": "C minor",
                "camelot": "10A",
                "energy": 8,
                "transition_note": "Peak energy, full mix",
                "source": "catalog",
                "track_id": "t3"
            }
        ],
        "notes": "A Middle Eastern journey from chill to peak energy"
    }"#
    .to_string()
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

// ---------------------------------------------------------------------------
// Integration Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_generate_round_trip() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(MockClaude {
        response: multi_track_llm_json(),
    });

    // POST /setlists/generate -> 201
    let (status, gen_json) = post_json(
        build_app(pool.clone(), claude.clone()),
        "/setlists/generate",
        serde_json::json!({ "prompt": "Middle Eastern journey" }),
    )
    .await;

    assert_eq!(status, 201);
    let setlist_id = gen_json["id"].as_str().unwrap();
    assert!(!setlist_id.is_empty());
    assert_eq!(gen_json["prompt"], "Middle Eastern journey");
    assert_eq!(gen_json["tracks"].as_array().unwrap().len(), 3);
    assert!(gen_json["harmonic_flow_score"].is_null()); // not yet arranged
                                                        // C1: score_breakdown should be null for generate
    assert!(gen_json["score_breakdown"].is_null());

    // GET /setlists/{id} -> 200 -> verify persisted
    let (status, get_json_resp) = get_json(
        build_app(pool.clone(), claude),
        &format!("/setlists/{setlist_id}"),
    )
    .await;

    assert_eq!(status, 200);
    assert_eq!(get_json_resp["id"], setlist_id);
    assert_eq!(get_json_resp["tracks"].as_array().unwrap().len(), 3);
    assert_eq!(get_json_resp["tracks"][0]["title"], "Desert Rose");
}

#[tokio::test]
async fn test_generate_arrange_round_trip() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(MockClaude {
        response: multi_track_llm_json(),
    });

    // Generate
    let (status, gen_json) = post_json(
        build_app(pool.clone(), claude.clone()),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;
    assert_eq!(status, 201);
    let setlist_id = gen_json["id"].as_str().unwrap();

    // Arrange
    let (status, arrange_json) = post_json(
        build_app(pool.clone(), claude.clone()),
        &format!("/setlists/{setlist_id}/arrange"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, 200);
    assert!(arrange_json["harmonic_flow_score"].as_f64().unwrap() > 0.0);
    assert_eq!(arrange_json["tracks"].as_array().unwrap().len(), 3);

    // C1: Verify score_breakdown is present after arrange
    assert!(arrange_json["score_breakdown"].is_object());
    assert!(arrange_json["score_breakdown"]["key_compatibility"]
        .as_f64()
        .is_some());
    assert!(arrange_json["score_breakdown"]["bpm_continuity"]
        .as_f64()
        .is_some());
    assert!(arrange_json["score_breakdown"]["energy_arc"]
        .as_f64()
        .is_some());

    // Verify transition scores present on non-first tracks
    let tracks = arrange_json["tracks"].as_array().unwrap();
    // First track has no transition score
    assert!(tracks[0]["transition_score"].is_null());
    // Subsequent tracks should have transition scores
    for track in tracks.iter().skip(1) {
        assert!(
            track["transition_score"].is_number(),
            "Track at position {} missing transition_score",
            track["position"]
        );
    }

    // GET after arrange -> verify scores persisted
    let (status, final_json) =
        get_json(build_app(pool, claude), &format!("/setlists/{setlist_id}")).await;
    assert_eq!(status, 200);
    assert!(final_json["harmonic_flow_score"].as_f64().is_some());
}

#[tokio::test]
async fn test_generate_with_claude_error() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(ErrorClaude);

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;

    assert_eq!(status, 503);
    assert!(json["error"].is_object());
    assert_eq!(json["error"]["code"], "LLM_ERROR");
}

// H1: Test rate limited -> SERVICE_BUSY (503)
#[tokio::test]
async fn test_generate_with_rate_limited_claude() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(RateLimitedClaude);

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;

    assert_eq!(status, 503);
    assert_eq!(json["error"]["code"], "SERVICE_BUSY");
}

// H1: Test timeout -> TIMEOUT (504)
#[tokio::test]
async fn test_generate_with_timeout_claude() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(TimeoutClaude);

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;

    assert_eq!(status, 504);
    assert_eq!(json["error"]["code"], "TIMEOUT");
}

#[tokio::test]
async fn test_arrange_preserves_track_count() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(MockClaude {
        response: multi_track_llm_json(),
    });

    // Generate
    let (_, gen_json) = post_json(
        build_app(pool.clone(), claude.clone()),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;
    let setlist_id = gen_json["id"].as_str().unwrap();
    let original_count = gen_json["tracks"].as_array().unwrap().len();

    // Arrange
    let (_, arrange_json) = post_json(
        build_app(pool, claude),
        &format!("/setlists/{setlist_id}/arrange"),
        serde_json::json!({}),
    )
    .await;

    assert_eq!(
        arrange_json["tracks"].as_array().unwrap().len(),
        original_count
    );
}

#[tokio::test]
async fn test_error_format_consistent() {
    let pool = create_test_pool().await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(MockClaude {
        response: "{}".to_string(),
    });

    // Empty prompt -> 400
    let (status, json) = post_json(
        build_app(pool.clone(), claude.clone()),
        "/setlists/generate",
        serde_json::json!({ "prompt": "" }),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"]["code"].is_string());
    assert!(json["error"]["message"].is_string());

    // Not found -> 404
    let (status, json) = get_json(build_app(pool, claude), "/setlists/nonexistent").await;
    assert_eq!(status, 404);
    assert!(json["error"]["code"].is_string());
    assert!(json["error"]["message"].is_string());
}

#[tokio::test]
async fn test_arrangement_deterministic() {
    let pool = create_test_pool().await;
    seed_tracks(&pool).await;
    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(MockClaude {
        response: multi_track_llm_json(),
    });

    // Generate a setlist
    let (_, gen_json) = post_json(
        build_app(pool.clone(), claude.clone()),
        "/setlists/generate",
        serde_json::json!({ "prompt": "test" }),
    )
    .await;
    let setlist_id = gen_json["id"].as_str().unwrap();

    // Arrange it
    let (_, first_arrange) = post_json(
        build_app(pool.clone(), claude.clone()),
        &format!("/setlists/{setlist_id}/arrange"),
        serde_json::json!({}),
    )
    .await;

    // Arrange again — should produce identical result
    let (_, second_arrange) = post_json(
        build_app(pool, claude),
        &format!("/setlists/{setlist_id}/arrange"),
        serde_json::json!({}),
    )
    .await;

    let first_score = first_arrange["harmonic_flow_score"].as_f64().unwrap();
    let second_score = second_arrange["harmonic_flow_score"].as_f64().unwrap();
    assert!(
        (first_score - second_score).abs() < 0.001,
        "Scores differ: {first_score} vs {second_score}"
    );
}

// M5: Integration test for arrange with 0 tracks
#[tokio::test]
async fn test_arrange_empty_setlist_returns_400() {
    let pool = create_test_pool().await;

    // Insert a setlist with no tracks
    sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES (?, ?, ?, ?)")
        .bind("empty-sl")
        .bind("user1")
        .bind("test")
        .bind("test-model")
        .execute(&pool)
        .await
        .unwrap();

    let claude: Arc<dyn ClaudeClientTrait> = Arc::new(MockClaude {
        response: "{}".to_string(),
    });

    let (status, json) = post_json(
        build_app(pool, claude),
        "/setlists/empty-sl/arrange",
        serde_json::json!({}),
    )
    .await;

    assert_eq!(status, 400);
    assert_eq!(json["error"]["code"], "INVALID_REQUEST");
}
