// ST-007: Quick commands (shuffle, sort-by-bpm, reverse, undo)

use rand::seq::SliceRandom;

use crate::db::models::SetlistTrackRow;

/// Deterministic operations on a setlist that do not require LLM.
#[derive(Debug, Clone, PartialEq)]
pub enum QuickCommand {
    Shuffle,
    SortByBpm,
    Reverse,
    Undo,
    RevertToVersion(i32),
}

/// Attempt to parse a user message as a quick command.
/// Returns None if it's not a recognized quick command (should go to LLM).
pub fn parse_quick_command(message: &str) -> Option<QuickCommand> {
    let s = message.trim().to_lowercase();
    match s.as_str() {
        "shuffle" => return Some(QuickCommand::Shuffle),
        "sort by bpm" | "sort-by-bpm" | "sort bpm" => return Some(QuickCommand::SortByBpm),
        "reverse" | "reverse order" => return Some(QuickCommand::Reverse),
        "undo" => return Some(QuickCommand::Undo),
        _ => {}
    }

    // "revert to version N"
    if let Some(rest) = s.strip_prefix("revert to version ") {
        if let Ok(n) = rest.trim().parse::<i32>() {
            return Some(QuickCommand::RevertToVersion(n));
        }
    }

    None
}

/// Shuffle tracks randomly. Re-numbers positions starting from 1.
/// Clears transition_note and transition_score since ordering changed.
pub fn apply_shuffle(tracks: &[SetlistTrackRow]) -> Vec<SetlistTrackRow> {
    let mut result: Vec<SetlistTrackRow> = tracks.to_vec();
    let mut rng = rand::thread_rng();
    result.shuffle(&mut rng);
    renumber_and_clear_transitions(&mut result);
    result
}

/// Sort tracks by BPM ascending. Tracks with no BPM go to the end.
/// Re-numbers positions. Clears transition notes.
pub fn apply_sort_by_bpm(tracks: &[SetlistTrackRow]) -> Vec<SetlistTrackRow> {
    let mut result: Vec<SetlistTrackRow> = tracks.to_vec();
    result.sort_by(|a, b| match (a.bpm, b.bpm) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    renumber_and_clear_transitions(&mut result);
    result
}

/// Reverse the track order. Re-numbers positions. Clears transition notes.
pub fn apply_reverse(tracks: &[SetlistTrackRow]) -> Vec<SetlistTrackRow> {
    let mut result: Vec<SetlistTrackRow> = tracks.iter().rev().cloned().collect();
    renumber_and_clear_transitions(&mut result);
    result
}

/// Re-number positions starting from 1 and clear transition data.
fn renumber_and_clear_transitions(tracks: &mut [SetlistTrackRow]) {
    for (i, track) in tracks.iter_mut().enumerate() {
        track.position = (i + 1) as i32;
        track.transition_note = None;
        track.transition_score = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_track(pos: i32, title: &str, bpm: Option<f64>) -> SetlistTrackRow {
        SetlistTrackRow {
            id: format!("st-{pos}"),
            setlist_id: "sl-1".to_string(),
            track_id: None,
            position: pos,
            original_position: pos,
            title: title.to_string(),
            artist: "Artist".to_string(),
            bpm,
            key: None,
            camelot: None,
            energy: None,
            transition_note: Some("blend".to_string()),
            transition_score: Some(80.0),
            source: "suggestion".to_string(),
            acquisition_info: None,
        }
    }

    // --- parse tests ---

    #[test]
    fn test_parse_shuffle() {
        assert_eq!(parse_quick_command("shuffle"), Some(QuickCommand::Shuffle));
        assert_eq!(parse_quick_command("SHUFFLE"), Some(QuickCommand::Shuffle));
        assert_eq!(
            parse_quick_command("  shuffle  "),
            Some(QuickCommand::Shuffle)
        );
    }

    #[test]
    fn test_parse_sort_by_bpm() {
        assert_eq!(
            parse_quick_command("sort by bpm"),
            Some(QuickCommand::SortByBpm)
        );
        assert_eq!(
            parse_quick_command("sort-by-bpm"),
            Some(QuickCommand::SortByBpm)
        );
        assert_eq!(
            parse_quick_command("sort bpm"),
            Some(QuickCommand::SortByBpm)
        );
        assert_eq!(
            parse_quick_command("SORT BY BPM"),
            Some(QuickCommand::SortByBpm)
        );
    }

    #[test]
    fn test_parse_reverse() {
        assert_eq!(parse_quick_command("reverse"), Some(QuickCommand::Reverse));
        assert_eq!(
            parse_quick_command("reverse order"),
            Some(QuickCommand::Reverse)
        );
        assert_eq!(parse_quick_command("REVERSE"), Some(QuickCommand::Reverse));
    }

    #[test]
    fn test_parse_undo() {
        assert_eq!(parse_quick_command("undo"), Some(QuickCommand::Undo));
        assert_eq!(parse_quick_command("UNDO"), Some(QuickCommand::Undo));
    }

    #[test]
    fn test_parse_revert() {
        assert_eq!(
            parse_quick_command("revert to version 3"),
            Some(QuickCommand::RevertToVersion(3))
        );
        assert_eq!(
            parse_quick_command("revert to version 1"),
            Some(QuickCommand::RevertToVersion(1))
        );
        assert_eq!(
            parse_quick_command("revert to version 0"),
            Some(QuickCommand::RevertToVersion(0))
        );
        assert_eq!(
            parse_quick_command("REVERT TO VERSION 7"),
            Some(QuickCommand::RevertToVersion(7))
        );
    }

    #[test]
    fn test_parse_not_quick_command() {
        assert_eq!(parse_quick_command("swap track 5"), None);
        assert_eq!(parse_quick_command("add something darker"), None);
        assert_eq!(parse_quick_command("hello"), None);
        assert_eq!(parse_quick_command("revert to version"), None);
        assert_eq!(parse_quick_command("revert to version abc"), None);
    }

    // --- apply tests ---

    #[test]
    fn test_apply_shuffle_preserves_track_count() {
        let tracks = vec![
            make_track(1, "Alpha", Some(120.0)),
            make_track(2, "Beta", Some(130.0)),
            make_track(3, "Gamma", None),
        ];
        let result = apply_shuffle(&tracks);
        assert_eq!(result.len(), tracks.len());
    }

    #[test]
    fn test_apply_shuffle_clears_transitions() {
        let tracks = vec![
            make_track(1, "Alpha", Some(120.0)),
            make_track(2, "Beta", Some(130.0)),
        ];
        let result = apply_shuffle(&tracks);
        for track in &result {
            assert!(track.transition_note.is_none());
            assert!(track.transition_score.is_none());
        }
    }

    #[test]
    fn test_apply_shuffle_renumbers_positions() {
        let tracks = vec![
            make_track(1, "Alpha", Some(120.0)),
            make_track(2, "Beta", Some(130.0)),
            make_track(3, "Gamma", None),
        ];
        let result = apply_shuffle(&tracks);
        let positions: Vec<i32> = result.iter().map(|t| t.position).collect();
        assert_eq!(positions, vec![1, 2, 3]);
    }

    #[test]
    fn test_apply_sort_by_bpm() {
        let tracks = vec![
            make_track(1, "Fast", Some(140.0)),
            make_track(2, "NoBpm", None),
            make_track(3, "Slow", Some(110.0)),
            make_track(4, "Mid", Some(125.0)),
        ];
        let result = apply_sort_by_bpm(&tracks);
        assert_eq!(result[0].title, "Slow");
        assert_eq!(result[1].title, "Mid");
        assert_eq!(result[2].title, "Fast");
        assert_eq!(result[3].title, "NoBpm");
    }

    #[test]
    fn test_apply_sort_by_bpm_positions_renumbered() {
        let tracks = vec![
            make_track(1, "Fast", Some(140.0)),
            make_track(2, "Slow", Some(110.0)),
        ];
        let result = apply_sort_by_bpm(&tracks);
        assert_eq!(result[0].position, 1);
        assert_eq!(result[1].position, 2);
        assert_eq!(result[0].title, "Slow");
    }

    #[test]
    fn test_apply_reverse() {
        let tracks = vec![
            make_track(1, "First", Some(120.0)),
            make_track(2, "Second", Some(125.0)),
            make_track(3, "Third", Some(130.0)),
        ];
        let result = apply_reverse(&tracks);
        assert_eq!(result[0].title, "Third");
        assert_eq!(result[1].title, "Second");
        assert_eq!(result[2].title, "First");
        assert_eq!(result[0].position, 1);
        assert_eq!(result[2].position, 3);
    }

    #[test]
    fn test_apply_reverse_clears_transitions() {
        let tracks = vec![
            make_track(1, "First", Some(120.0)),
            make_track(2, "Second", Some(125.0)),
        ];
        let result = apply_reverse(&tracks);
        for track in &result {
            assert!(track.transition_note.is_none());
            assert!(track.transition_score.is_none());
        }
    }

    #[test]
    fn test_empty_tracks() {
        let empty: Vec<SetlistTrackRow> = vec![];
        assert!(apply_shuffle(&empty).is_empty());
        assert!(apply_sort_by_bpm(&empty).is_empty());
        assert!(apply_reverse(&empty).is_empty());
    }
}
