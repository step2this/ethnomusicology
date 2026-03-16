use axum::body::Body;
use axum::http::Request;
use sqlx::PgPool;
use tower::ServiceExt;

async fn create_test_pool() -> PgPool {
    // Use the shared migration runner — single source of truth in db/mod.rs
    ethnomusicology_backend::db::create_test_pool().await
}

fn build_app(pool: PgPool) -> axum::Router {
    ethnomusicology_backend::routes::tracks::tracks_router(pool)
}

async fn get_json(app: axum::Router, uri: &str) -> (u16, serde_json::Value) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

async fn seed_track(pool: &PgPool, id: &str, title: &str, bpm: Option<f64>) {
    let mut q = "INSERT INTO tracks (id, title, source".to_string();
    if bpm.is_some() {
        q.push_str(", bpm");
    }
    q.push_str(") VALUES ($1, $2, $3");
    if bpm.is_some() {
        q.push_str(", $4");
    }
    q.push(')');

    let mut query = sqlx::query(&q).bind(id).bind(title).bind("spotify");
    if let Some(b) = bpm {
        query = query.bind(b);
    }
    query.execute(pool).await.unwrap();
}

async fn seed_artist(pool: &PgPool, id: &str, name: &str) {
    sqlx::query("INSERT INTO artists (id, name) VALUES ($1, $2)")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
}

async fn link_track_artist(pool: &PgPool, track_id: &str, artist_id: &str) {
    sqlx::query("INSERT INTO track_artists (track_id, artist_id) VALUES ($1, $2)")
        .bind(track_id)
        .bind(artist_id)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_round_trip_basic() {
    let pool = create_test_pool().await;

    seed_track(&pool, "t1", "Gnawa Blues", None).await;
    seed_track(&pool, "t2", "Desert Rose", None).await;
    seed_track(&pool, "t3", "Sufi Trance", None).await;

    seed_artist(&pool, "a1", "Hassan Hakmoun").await;
    link_track_artist(&pool, "t1", "a1").await;

    let (status, json) = get_json(build_app(pool), "/tracks").await;
    assert_eq!(status, 200);
    assert_eq!(json["data"].as_array().unwrap().len(), 3);
    assert_eq!(json["total"], 3);

    // Verify nullable fields are null for Spotify-imported tracks without DJ metadata
    let first_track = &json["data"][0];
    assert!(first_track["bpm"].is_null());
    assert!(first_track["key"].is_null());
    assert!(first_track["energy"].is_null());
    assert!(first_track["album_art_url"].is_null());
}

#[tokio::test]
async fn test_round_trip_pagination() {
    let pool = create_test_pool().await;

    for i in 0..7 {
        seed_track(&pool, &format!("t{i}"), &format!("Track {i}"), None).await;
    }

    let app = build_app(pool.clone());
    let (status, json) = get_json(app, "/tracks?page=1&per_page=3").await;
    assert_eq!(status, 200);
    assert_eq!(json["data"].as_array().unwrap().len(), 3);
    assert_eq!(json["total"], 7);
    assert_eq!(json["total_pages"], 3);

    let app2 = build_app(pool.clone());
    let (_, json2) = get_json(app2, "/tracks?page=2&per_page=3").await;
    assert_eq!(json2["data"].as_array().unwrap().len(), 3);

    // Verify no overlap between pages
    let page1_ids: Vec<&str> = json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    let page2_ids: Vec<&str> = json2["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    for id in &page1_ids {
        assert!(
            !page2_ids.contains(id),
            "Track {} appears on both pages",
            id
        );
    }

    let app3 = build_app(pool);
    let (_, json3) = get_json(app3, "/tracks?page=3&per_page=3").await;
    assert_eq!(json3["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_round_trip_sorting() {
    let pool = create_test_pool().await;

    seed_track(&pool, "t1", "Slow", Some(100.0)).await;
    seed_track(&pool, "t2", "Fast", Some(128.0)).await;
    seed_track(&pool, "t3", "Unknown", None).await;

    let app = build_app(pool.clone());
    let (_, json) = get_json(app, "/tracks?sort=bpm&order=asc").await;
    let tracks = json["data"].as_array().unwrap();
    assert_eq!(tracks[0]["bpm"], 100.0);
    assert_eq!(tracks[1]["bpm"], 128.0);
    assert!(tracks[2]["bpm"].is_null());

    let app2 = build_app(pool);
    let (_, json2) = get_json(app2, "/tracks?sort=bpm&order=desc").await;
    let tracks2 = json2["data"].as_array().unwrap();
    assert_eq!(tracks2[0]["bpm"], 128.0);
    assert_eq!(tracks2[1]["bpm"], 100.0);
    assert!(tracks2[2]["bpm"].is_null());
}

#[tokio::test]
async fn test_round_trip_error_format() {
    let pool = create_test_pool().await;
    let (status, json) = get_json(build_app(pool), "/tracks?page=0").await;
    assert_eq!(status, 400);
    assert_eq!(json["error"]["code"], "INVALID_REQUEST");
    assert!(!json["error"]["message"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_round_trip_multi_artist() {
    let pool = create_test_pool().await;

    seed_track(&pool, "t1", "Collab", None).await;
    seed_artist(&pool, "a1", "Hassan Hakmoun").await;
    seed_artist(&pool, "a2", "Gnawa Diffusion").await;
    link_track_artist(&pool, "t1", "a1").await;
    link_track_artist(&pool, "t1", "a2").await;

    let (_, json) = get_json(build_app(pool), "/tracks").await;
    let artist = json["data"][0]["artist"].as_str().unwrap();
    assert!(
        artist.contains("Hassan Hakmoun"),
        "Expected artist string to contain 'Hassan Hakmoun', got: {}",
        artist
    );
    assert!(
        artist.contains("Gnawa Diffusion"),
        "Expected artist string to contain 'Gnawa Diffusion', got: {}",
        artist
    );
}

#[tokio::test]
async fn test_round_trip_empty_catalog() {
    let pool = create_test_pool().await;
    let (status, json) = get_json(build_app(pool), "/tracks").await;
    assert_eq!(status, 200);
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
    assert_eq!(json["total"], 0);
    assert_eq!(json["total_pages"], 0);
}
