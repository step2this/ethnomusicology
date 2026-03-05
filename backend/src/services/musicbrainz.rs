use std::sync::OnceLock;
use std::time::Duration;

use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::services::match_scoring::is_acceptable_match;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct MusicBrainzMatch {
    pub title: String,
    pub artist: String,
    pub isrc: Option<String>,
    pub score: f64,
    pub musicbrainz_id: String,
}

// ---------------------------------------------------------------------------
// Internal deserialisation types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct MbResponse {
    recordings: Vec<MbRecording>,
}

#[derive(Deserialize)]
struct MbRecording {
    id: String,
    score: Option<u32>,
    title: String,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<MbArtistCredit>>,
    isrcs: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct MbArtistCredit {
    artist: MbArtist,
}

#[derive(Deserialize)]
struct MbArtist {
    name: String,
}

// ---------------------------------------------------------------------------
// Rate limiter — MusicBrainz allows 1 request/second for anonymous clients
// ---------------------------------------------------------------------------

static MB_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

fn mb_semaphore() -> &'static Semaphore {
    MB_SEMAPHORE.get_or_init(|| Semaphore::new(1))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Search MusicBrainz for a recording matching the given artist and title.
///
/// Returns the best acceptable match (title similarity ≥ 0.5 and artist
/// similarity ≥ 0.5 per `is_acceptable_match`), or `None` if no match is
/// found.
///
/// Uses a global semaphore with 1 permit to stay within MusicBrainz's
/// anonymous rate limit of 1 request/second.
pub async fn search_recording(
    client: &reqwest::Client,
    artist: &str,
    title: &str,
) -> Option<MusicBrainzMatch> {
    let _permit = mb_semaphore().acquire().await.ok()?;

    // Escape Lucene special chars in artist/title before building query
    fn escape_lucene(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
    }
    let query = format!(
        "artist:\"{}\" AND recording:\"{}\"",
        escape_lucene(artist),
        escape_lucene(title),
    );

    let resp = tokio::time::timeout(
        Duration::from_secs(5),
        client
            .get("https://musicbrainz.org/ws/2/recording")
            .query(&[("query", &query), ("fmt", &"json".to_string()), ("limit", &"5".to_string())])
            .header(
                reqwest::header::USER_AGENT,
                "tarab-studio/0.1.0 (https://tarab.studio)",
            )
            .send(),
    )
    .await
    .ok()?
    .ok()?;

    // Rate limit: sleep 1s after each request to respect MusicBrainz's 1 req/sec policy
    tokio::time::sleep(Duration::from_secs(1)).await;

    if !resp.status().is_success() {
        return None;
    }

    let mb_resp: MbResponse = resp.json().await.ok()?;

    mb_resp.recordings.into_iter().find_map(|rec| {
        // Extract artist name as owned String before any moves on rec.
        let result_artist = rec
            .artist_credit
            .as_deref()
            .and_then(|credits| credits.first())
            .map(|ac| ac.artist.name.clone())
            .unwrap_or_default();

        if !is_acceptable_match(title, artist, &rec.title, &result_artist) {
            return None;
        }

        let isrc = rec.isrcs.and_then(|v| v.into_iter().next());
        let score = rec.score.unwrap_or(0) as f64;

        Some(MusicBrainzMatch {
            title: rec.title,
            artist: result_artist,
            isrc,
            score,
            musicbrainz_id: rec.id,
        })
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musicbrainz_match_struct() {
        let m = MusicBrainzMatch {
            title: "Strings of Life".to_string(),
            artist: "Rhythim Is Rhythim".to_string(),
            isrc: Some("USRC11234567".to_string()),
            score: 100.0,
            musicbrainz_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        };
        assert_eq!(m.title, "Strings of Life");
        assert_eq!(m.artist, "Rhythim Is Rhythim");
        assert_eq!(m.isrc.as_deref(), Some("USRC11234567"));
        assert_eq!(m.score, 100.0);
        assert_eq!(m.musicbrainz_id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_semaphore_is_singleton() {
        let sem1 = mb_semaphore();
        let sem2 = mb_semaphore();
        // Both calls return the same static instance
        assert!(std::ptr::eq(sem1, sem2));
    }

    #[test]
    fn mb_response_deserialises() {
        let json = r#"{
            "recordings": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "score": 100,
                    "title": "Strings of Life",
                    "artist-credit": [
                        {
                            "artist": {
                                "id": "artist-uuid",
                                "name": "Rhythim Is Rhythim"
                            }
                        }
                    ],
                    "isrcs": ["USRC11234567"],
                    "releases": []
                }
            ]
        }"#;
        let resp: MbResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.recordings.len(), 1);
        let rec = &resp.recordings[0];
        assert_eq!(rec.id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(rec.title, "Strings of Life");
        assert_eq!(rec.score, Some(100));
        let artist_name = &rec
            .artist_credit
            .as_ref()
            .unwrap()
            .first()
            .unwrap()
            .artist
            .name;
        assert_eq!(artist_name, "Rhythim Is Rhythim");
        assert_eq!(rec.isrcs.as_ref().unwrap()[0], "USRC11234567");
    }

    #[test]
    fn mb_response_handles_missing_optional_fields() {
        let json = r#"{
            "recordings": [
                {
                    "id": "some-uuid",
                    "title": "Unknown Track"
                }
            ]
        }"#;
        let resp: MbResponse = serde_json::from_str(json).unwrap();
        let rec = &resp.recordings[0];
        assert!(rec.score.is_none());
        assert!(rec.artist_credit.is_none());
        assert!(rec.isrcs.is_none());
    }
}
