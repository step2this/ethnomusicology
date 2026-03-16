use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use sqlx::PgPool;
use tower::ServiceExt;

use ethnomusicology_backend::api::claude::{
    CacheMetrics, ClaudeClientTrait, ClaudeError, ConversationMessage, RequestContentBlock,
};
use ethnomusicology_backend::routes::refinement::{refinement_router, RefinementRouteState};

// ---------------------------------------------------------------------------
// Mock Claude — returns a canned converse() response
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
        Ok(String::new())
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Ok((String::new(), CacheMetrics::default()))
    }

    async fn converse(
        &self,
        _system_prompt: &str,
        _messages: Vec<ConversationMessage>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Ok(self.response.clone())
    }
}

// ---------------------------------------------------------------------------
// PanickingClaude — asserts converse() is never called (quick command tests)
// ---------------------------------------------------------------------------

struct PanickingClaude;

#[async_trait::async_trait]
impl ClaudeClientTrait for PanickingClaude {
    async fn generate_setlist(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        Ok(String::new())
    }

    async fn generate_with_blocks(
        &self,
        _system_blocks: Vec<RequestContentBlock>,
        _user_blocks: Vec<RequestContentBlock>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<(String, CacheMetrics), ClaudeError> {
        Ok((String::new(), CacheMetrics::default()))
    }

    async fn converse(
        &self,
        _system_prompt: &str,
        _messages: Vec<ConversationMessage>,
        _model: &str,
        _max_tokens: u32,
    ) -> Result<String, ClaudeError> {
        panic!("converse() must not be called for quick commands")
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn create_test_pool() -> PgPool {
    ethnomusicology_backend::db::create_test_pool().await
}

async fn insert_setlist(pool: &PgPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO setlists (id, user_id, prompt, model) VALUES ($1, $2, $3, $4)")
        .bind(&id)
        .bind("anonymous")
        .bind("test prompt")
        .bind("claude-test")
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn insert_setlist_tracks(pool: &PgPool, setlist_id: &str, count: usize) {
    for i in 1..=count {
        let track_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO setlist_tracks \
             (id, setlist_id, position, original_position, title, artist, bpm, key, camelot, source) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&track_id)
        .bind(setlist_id)
        .bind(i as i32)
        .bind(i as i32)
        .bind(format!("Track {i}"))
        .bind("Artist")
        .bind(120.0 + i as f64)
        .bind(format!("A{i}"))
        .bind(format!("{}A", i % 12 + 1))
        .bind("suggestion")
        .execute(pool)
        .await
        .unwrap();
    }
}

fn build_app(pool: PgPool, claude: impl ClaudeClientTrait + 'static) -> Router {
    let state = Arc::new(RefinementRouteState {
        pool,
        claude: Arc::new(claude),
    });
    refinement_router(state)
}

fn replace_response(position: usize, title: &str, artist: &str) -> String {
    serde_json::json!({
        "actions": [{
            "type": "replace",
            "position": position,
            "title": title,
            "artist": artist,
            "bpm": null,
            "key": null
        }],
        "explanation": format!("Replaced track {position} with {title}")
    })
    .to_string()
}

async fn post_json(app: Router, uri: &str, body: serde_json::Value) -> (u16, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body_bytes).unwrap())
}

async fn post_empty(app: Router, uri: &str) -> (u16, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body_bytes).unwrap())
}

async fn get_json(app: Router, uri: &str) -> (u16, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body_bytes).unwrap())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_refine_round_trip() {
    let pool = create_test_pool().await;
    let setlist_id = insert_setlist(&pool).await;
    insert_setlist_tracks(&pool, &setlist_id, 2).await;

    let app = build_app(
        pool,
        MockClaude {
            response: replace_response(1, "New Track", "New Artist"),
        },
    );

    let (status, json) = post_json(
        app,
        &format!("/setlists/{setlist_id}/refine"),
        serde_json::json!({"message": "swap track 1 for something darker"}),
    )
    .await;

    assert_eq!(status, 200);
    assert_eq!(json["version_number"], 1);
    let tracks = json["tracks"].as_array().unwrap();
    assert!(!tracks.is_empty());
    assert_eq!(json["tracks"][0]["title"], "New Track");
    assert!(!json["explanation"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_refine_quick_command_shuffle_no_llm() {
    let pool = create_test_pool().await;
    let setlist_id = insert_setlist(&pool).await;
    insert_setlist_tracks(&pool, &setlist_id, 3).await;

    // PanickingClaude ensures converse() is never called for quick commands
    let app = build_app(pool, PanickingClaude);

    let (status, json) = post_json(
        app,
        &format!("/setlists/{setlist_id}/refine"),
        serde_json::json!({"message": "shuffle"}),
    )
    .await;

    assert_eq!(status, 200);
    let tracks = json["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 3);
}

#[tokio::test]
async fn test_revert_to_version() {
    let pool = create_test_pool().await;
    let setlist_id = insert_setlist(&pool).await;
    insert_setlist_tracks(&pool, &setlist_id, 2).await;

    // Step 1: Refine → creates v0 (bootstrap) + v1 (refined)
    let app1 = build_app(
        pool.clone(),
        MockClaude {
            response: replace_response(1, "Refined Track", "Refined Artist"),
        },
    );
    let (status, _) = post_json(
        app1,
        &format!("/setlists/{setlist_id}/refine"),
        serde_json::json!({"message": "change track 1"}),
    )
    .await;
    assert_eq!(status, 200);

    // Step 2: Revert to version 0 (bootstrap — has original tracks)
    let app2 = build_app(
        pool.clone(),
        MockClaude {
            response: String::new(),
        },
    );
    let (status, json) = post_empty(app2, &format!("/setlists/{setlist_id}/revert/0")).await;

    assert_eq!(status, 200);
    // After reverting to v0, the tracks should be the original ones
    assert_eq!(json["tracks"][0]["title"], "Track 1");
}

#[tokio::test]
async fn test_history_returns_versions_and_conversations() {
    let pool = create_test_pool().await;
    let setlist_id = insert_setlist(&pool).await;
    insert_setlist_tracks(&pool, &setlist_id, 2).await;

    // Refine once
    let app1 = build_app(
        pool.clone(),
        MockClaude {
            response: replace_response(1, "First Refined", "Artist A"),
        },
    );
    let (status, _) = post_json(
        app1,
        &format!("/setlists/{setlist_id}/refine"),
        serde_json::json!({"message": "change track 1"}),
    )
    .await;
    assert_eq!(status, 200);

    // Refine again
    let app2 = build_app(
        pool.clone(),
        MockClaude {
            response: replace_response(2, "Second Refined", "Artist B"),
        },
    );
    let (status, _) = post_json(
        app2,
        &format!("/setlists/{setlist_id}/refine"),
        serde_json::json!({"message": "change track 2"}),
    )
    .await;
    assert_eq!(status, 200);

    // Get history
    let app3 = build_app(
        pool.clone(),
        MockClaude {
            response: String::new(),
        },
    );
    let (status, json) = get_json(app3, &format!("/setlists/{setlist_id}/history")).await;

    assert_eq!(status, 200);
    let versions = json["versions"].as_array().unwrap();
    let conversations = json["conversations"].as_array().unwrap();
    // v0 (bootstrap) + v1 + v2 = at least 3 versions
    assert!(versions.len() >= 3);
    // 2 user + 2 assistant messages
    assert!(conversations.len() >= 4);
}

#[tokio::test]
async fn test_refine_not_found_returns_404() {
    let pool = create_test_pool().await;
    let app = build_app(
        pool,
        MockClaude {
            response: String::new(),
        },
    );

    let (status, json) = post_json(
        app,
        "/setlists/nonexistent-setlist-id/refine",
        serde_json::json!({"message": "change something"}),
    )
    .await;

    assert_eq!(status, 404);
    assert_eq!(json["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_refine_empty_message_returns_400() {
    let pool = create_test_pool().await;
    let setlist_id = insert_setlist(&pool).await;
    insert_setlist_tracks(&pool, &setlist_id, 2).await;

    let app = build_app(
        pool,
        MockClaude {
            response: String::new(),
        },
    );

    let (status, json) = post_json(
        app,
        &format!("/setlists/{setlist_id}/refine"),
        serde_json::json!({"message": ""}),
    )
    .await;

    assert_eq!(status, 400);
    assert_eq!(json["error"]["code"], "INVALID_REQUEST");
}
