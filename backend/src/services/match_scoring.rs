/// Strip parenthetical content like "(feat. ...)", "(remix)", "(original mix)", etc.
fn strip_noise(s: &str) -> String {
    let s = s.to_lowercase();
    // Remove everything inside parentheses (including the parens)
    let mut result = String::new();
    let mut depth = 0usize;
    for ch in s.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
            }
            _ => {
                if depth == 0 {
                    result.push(ch);
                }
            }
        }
    }
    result.trim().to_string()
}

/// Normalized similarity score between two strings (0.0 to 1.0).
/// Uses case-insensitive substring and word-overlap matching.
pub fn title_similarity(query: &str, result: &str) -> f64 {
    let q = strip_noise(query);
    let r = strip_noise(result);

    if q == r {
        return 1.0;
    }

    if q.is_empty() || r.is_empty() {
        return 0.0;
    }

    // Substring containment
    if q.contains(r.as_str()) || r.contains(q.as_str()) {
        // Score based on length ratio so "a" vs "a very long string" isn't 0.8
        let shorter = q.len().min(r.len()) as f64;
        let longer = q.len().max(r.len()) as f64;
        let ratio = shorter / longer;
        // Give at least 0.8 if there's a strong substring relationship
        return if ratio >= 0.5 {
            0.8 + 0.2 * ratio
        } else {
            0.8 * ratio + 0.4
        };
    }

    // Word overlap scoring
    let q_words: Vec<&str> = q.split_whitespace().collect();
    let r_words: Vec<&str> = r.split_whitespace().collect();

    if q_words.is_empty() || r_words.is_empty() {
        return 0.0;
    }

    let (query_words, result_words) = if q_words.len() <= r_words.len() {
        (&q_words, &r_words)
    } else {
        (&r_words, &q_words)
    };

    let matching = query_words
        .iter()
        .filter(|w| result_words.contains(w))
        .count();

    if matching == 0 {
        return 0.0;
    }

    // Score = matching words / total words in the shorter (query) string
    matching as f64 / query_words.len() as f64
}

pub fn artist_similarity(query: &str, result: &str) -> f64 {
    title_similarity(query, result)
}

/// Returns true if the match is acceptable (score >= 0.5 for both title and artist).
pub fn is_acceptable_match(
    query_title: &str,
    query_artist: &str,
    result_title: &str,
    result_artist: &str,
) -> bool {
    title_similarity(query_title, result_title) >= 0.5
        && artist_similarity(query_artist, result_artist) >= 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert_eq!(title_similarity("Levels", "Levels"), 1.0);
    }

    #[test]
    fn case_insensitive() {
        assert!(title_similarity("levels", "Levels") >= 0.9);
    }

    #[test]
    fn substring_match() {
        assert!(title_similarity("Levels", "Levels (Original Mix)") >= 0.7);
    }

    #[test]
    fn no_match() {
        assert!(title_similarity("Completely Different", "Nothing Similar") < 0.5);
    }

    #[test]
    fn acceptable_match() {
        assert!(is_acceptable_match("Levels", "Avicii", "Levels", "Avicii"));
    }

    #[test]
    fn unacceptable_artist() {
        assert!(!is_acceptable_match(
            "Levels",
            "Avicii",
            "Levels",
            "Some Other Artist"
        ));
    }

    #[test]
    fn parenthetical_stripped_before_compare() {
        // "Levels" vs "Levels (Original Mix)" — after stripping, "levels" == "levels"
        assert_eq!(title_similarity("Levels", "Levels (Original Mix)"), 1.0);
    }

    #[test]
    fn feat_stripped() {
        assert!(title_similarity("Song (feat. Artist B)", "Song") >= 0.9);
    }

    #[test]
    fn word_overlap_partial() {
        let score = title_similarity("Summer Love", "Summer Love In Paris");
        assert!(score >= 0.5, "score was {score}");
    }

    #[test]
    fn completely_empty_query() {
        assert_eq!(title_similarity("", "Levels"), 0.0);
    }

    #[test]
    fn both_empty() {
        // Both empty after stripping → both equal → 1.0
        assert_eq!(title_similarity("", ""), 1.0);
    }

    #[test]
    fn artist_similarity_mirrors_title() {
        assert_eq!(
            artist_similarity("Avicii", "Avicii"),
            title_similarity("Avicii", "Avicii")
        );
    }

    #[test]
    fn unacceptable_title() {
        assert!(!is_acceptable_match(
            "Levels",
            "Avicii",
            "Wake Me Up",
            "Avicii"
        ));
    }

    #[test]
    fn single_word_substring() {
        // "Love" is a substring of "Endless Love"
        let score = title_similarity("Love", "Endless Love");
        assert!(score >= 0.4, "score was {score}");
    }
}
