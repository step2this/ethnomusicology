use serde::Serialize;

// ---------------------------------------------------------------------------
// Camelot Key
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CamelotKey {
    pub number: u8,   // 1-12
    pub letter: char, // A or B
}

impl std::fmt::Display for CamelotKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.number, self.letter)
    }
}

/// Parse a Camelot notation string (e.g., "8A", "11B") into a CamelotKey.
pub fn parse_camelot(notation: &str) -> Option<CamelotKey> {
    let notation = notation.trim();
    if notation.is_empty() {
        return None;
    }

    let letter = notation.chars().last()?;
    if letter != 'A' && letter != 'B' {
        return None;
    }

    let number_str = &notation[..notation.len() - 1];
    let number: u8 = number_str.parse().ok()?;
    if !(1..=12).contains(&number) {
        return None;
    }

    Some(CamelotKey { number, letter })
}

// ---------------------------------------------------------------------------
// Scoring Functions
// ---------------------------------------------------------------------------

/// Key compatibility score between two Camelot keys based on the Camelot wheel.
///
/// The Camelot wheel defines three standard DJ-compatible key transitions:
///
/// 1. **Same key** (e.g., 8A -> 8A): Perfect match, score = 1.0
/// 2. **Adjacent on the wheel** (same letter, ±1 number, e.g., 8A -> 9A):
///    Moving one step around the wheel's outer (A) or inner (B) ring. Score = 0.9
/// 3. **Relative major/minor** (same number, cross letter, e.g., 8A -> 8B):
///    Switching between the outer and inner ring at the same position. Score = 0.9
///
/// Same-letter ±2 numbers (e.g., 8A -> 10A) is a tolerable but noticeable key
/// shift, scored at 0.5 as a moderate compatibility.
///
/// All other transitions — including cross-letter with different numbers
/// (e.g., 8A -> 9B) — return 0.0. This is intentional: the Camelot wheel only
/// guarantees smooth mixing along its three standard transitions. Combining a
/// cross-letter and different-number move produces two incompatible shifts and
/// is not a recognized DJ-safe transition.
pub fn camelot_score(a: &CamelotKey, b: &CamelotKey) -> f64 {
    if a.number == b.number && a.letter == b.letter {
        return 1.0; // Transition 1: Same key
    }

    // Transition 3: Relative major/minor (same number, cross letter)
    if a.number == b.number && a.letter != b.letter {
        return 0.9;
    }

    // Transition 2 (and ±2 tolerance): Same letter, check wheel distance
    if a.letter == b.letter {
        let diff = wheel_distance(a.number, b.number);
        if diff == 1 {
            return 0.9; // Adjacent on wheel
        }
        if diff == 2 {
            return 0.5; // Tolerable shift
        }
    }

    // All other combinations (including cross-letter + different-number) are
    // incompatible per Camelot wheel rules.
    0.0
}

/// BPM compatibility score.
/// 0-2 → 1.0, 3-4 → 0.8, 5-6 → 0.6, 7-10 → 0.3, >10 → 0.0
pub fn bpm_score(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs();
    if diff <= 2.0 {
        1.0
    } else if diff <= 4.0 {
        0.8
    } else if diff <= 6.0 {
        0.6
    } else if diff <= 10.0 {
        0.3
    } else {
        0.0
    }
}

/// Energy arc score: rewards build/peak/cooldown progression.
/// Position-based: first third = build (low energy), middle = peak, last third = cooldown.
pub fn energy_arc_score(energy: i32, position: usize, total: usize) -> f64 {
    if total <= 1 {
        return 1.0;
    }

    let fraction = position as f64 / (total - 1) as f64;
    let ideal_energy = if fraction < 0.33 {
        // Build phase: energy rises from 3 to 7
        3.0 + (fraction / 0.33) * 4.0
    } else if fraction < 0.67 {
        // Peak phase: energy around 7-9
        7.0 + ((fraction - 0.33) / 0.34) * 2.0
    } else {
        // Cooldown phase: energy drops from 9 to 4
        9.0 - ((fraction - 0.67) / 0.33) * 5.0
    };

    let diff = (energy as f64 - ideal_energy).abs();
    if diff <= 1.0 {
        1.0
    } else if diff <= 2.0 {
        0.8
    } else if diff <= 3.0 {
        0.5
    } else {
        0.2
    }
}

/// Neutral energy score used in pair-wise transition scoring.
///
/// Energy scoring is intentionally handled as a *positional* concern by
/// [`energy_arc_score`] in the arrangement algorithm, not as a pair-wise
/// transition property. A neutral value is used here so that energy neither
/// rewards nor penalizes any individual transition.
///
/// As a consequence, the theoretical maximum `transition_score` is 0.9
/// (key=1.0*0.5 + bpm=1.0*0.3 + energy=0.5*0.2 = 0.5 + 0.3 + 0.1), not 1.0.
const ENERGY_NEUTRAL: f64 = 0.5;

/// Transition score combining key, BPM, and energy factors.
/// Weights: key 50%, BPM 30%, energy 20%.
///
/// The energy component always uses [`ENERGY_NEUTRAL`] (0.5) because energy
/// scoring is positional — it is evaluated per-track by [`energy_arc_score`]
/// during arrangement, not between adjacent pairs. This means the theoretical
/// maximum transition score is **0.9**, not 1.0.
pub fn transition_score(
    camelot_a: Option<&CamelotKey>,
    camelot_b: Option<&CamelotKey>,
    bpm_a: Option<f64>,
    bpm_b: Option<f64>,
) -> f64 {
    let key_score = match (camelot_a, camelot_b) {
        (Some(a), Some(b)) => camelot_score(a, b),
        _ => 0.5, // neutral when missing
    };

    let bpm_s = match (bpm_a, bpm_b) {
        (Some(a), Some(b)) => bpm_score(a, b),
        _ => 0.5, // neutral when missing
    };

    key_score * 0.5 + bpm_s * 0.3 + ENERGY_NEUTRAL * 0.2
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Distance on the Camelot wheel (wraps 12↔1).
fn wheel_distance(a: u8, b: u8) -> u8 {
    let diff = (a as i8 - b as i8).unsigned_abs();
    diff.min(12 - diff)
}

/// Score breakdown for arrangement results.
#[derive(Debug, Clone, Serialize)]
pub struct ScoreBreakdown {
    pub key_compatibility: f64,
    pub bpm_continuity: f64,
    pub energy_arc: f64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_camelot ---

    #[test]
    fn test_parse_valid_camelot_keys() {
        assert_eq!(
            parse_camelot("1A"),
            Some(CamelotKey {
                number: 1,
                letter: 'A'
            })
        );
        assert_eq!(
            parse_camelot("12B"),
            Some(CamelotKey {
                number: 12,
                letter: 'B'
            })
        );
        assert_eq!(
            parse_camelot("8A"),
            Some(CamelotKey {
                number: 8,
                letter: 'A'
            })
        );
    }

    #[test]
    fn test_parse_camelot_with_whitespace() {
        assert_eq!(
            parse_camelot(" 8A "),
            Some(CamelotKey {
                number: 8,
                letter: 'A'
            })
        );
    }

    #[test]
    fn test_parse_invalid_camelot() {
        assert!(parse_camelot("").is_none());
        assert!(parse_camelot("0A").is_none());
        assert!(parse_camelot("13A").is_none());
        assert!(parse_camelot("8C").is_none());
        assert!(parse_camelot("AB").is_none());
        assert!(parse_camelot("A").is_none());
    }

    // --- camelot_score ---

    #[test]
    fn test_camelot_score_same_key() {
        let a = CamelotKey {
            number: 8,
            letter: 'A',
        };
        assert_eq!(camelot_score(&a, &a), 1.0);
    }

    #[test]
    fn test_camelot_score_adjacent() {
        let a = CamelotKey {
            number: 8,
            letter: 'A',
        };
        let b = CamelotKey {
            number: 9,
            letter: 'A',
        };
        assert_eq!(camelot_score(&a, &b), 0.9);
    }

    #[test]
    fn test_camelot_score_cross_letter() {
        let a = CamelotKey {
            number: 8,
            letter: 'A',
        };
        let b = CamelotKey {
            number: 8,
            letter: 'B',
        };
        assert_eq!(camelot_score(&a, &b), 0.9);
    }

    #[test]
    fn test_camelot_score_two_apart() {
        let a = CamelotKey {
            number: 8,
            letter: 'A',
        };
        let b = CamelotKey {
            number: 10,
            letter: 'A',
        };
        assert_eq!(camelot_score(&a, &b), 0.5);
    }

    #[test]
    fn test_camelot_score_distant() {
        let a = CamelotKey {
            number: 1,
            letter: 'A',
        };
        let b = CamelotKey {
            number: 7,
            letter: 'A',
        };
        assert_eq!(camelot_score(&a, &b), 0.0);
    }

    #[test]
    fn test_camelot_score_wraps_12_to_1() {
        let a = CamelotKey {
            number: 12,
            letter: 'A',
        };
        let b = CamelotKey {
            number: 1,
            letter: 'A',
        };
        assert_eq!(camelot_score(&a, &b), 0.9); // adjacent on wheel
    }

    #[test]
    fn test_camelot_score_wraps_11_to_1() {
        let a = CamelotKey {
            number: 11,
            letter: 'A',
        };
        let b = CamelotKey {
            number: 1,
            letter: 'A',
        };
        assert_eq!(camelot_score(&a, &b), 0.5); // 2 apart on wheel
    }

    // --- bpm_score ---

    #[test]
    fn test_bpm_score_close() {
        assert_eq!(bpm_score(128.0, 128.0), 1.0);
        assert_eq!(bpm_score(128.0, 130.0), 1.0);
    }

    #[test]
    fn test_bpm_score_moderate() {
        assert_eq!(bpm_score(128.0, 132.0), 0.8);
    }

    #[test]
    fn test_bpm_score_far() {
        assert_eq!(bpm_score(128.0, 134.0), 0.6);
    }

    #[test]
    fn test_bpm_score_very_far() {
        assert_eq!(bpm_score(128.0, 138.0), 0.3);
    }

    #[test]
    fn test_bpm_score_incompatible() {
        assert_eq!(bpm_score(128.0, 145.0), 0.0);
    }

    // --- energy_arc_score ---

    #[test]
    fn test_energy_arc_single_track() {
        assert_eq!(energy_arc_score(5, 0, 1), 1.0);
    }

    #[test]
    fn test_energy_arc_build_phase() {
        // Position 0 of 10: ideal ~3. Low energy=3 should score well.
        let score = energy_arc_score(3, 0, 10);
        assert!(
            score >= 0.8,
            "Build phase low energy should score well: {score}"
        );
    }

    #[test]
    fn test_energy_arc_peak_phase() {
        // Position 5 of 10: ideal ~8. High energy should score well.
        let score = energy_arc_score(8, 5, 10);
        assert!(
            score >= 0.8,
            "Peak phase high energy should score well: {score}"
        );
    }

    #[test]
    fn test_energy_arc_cooldown_phase() {
        // Position 9 of 10: ideal ~4. Medium energy should score well.
        let score = energy_arc_score(4, 9, 10);
        assert!(
            score >= 0.8,
            "Cooldown phase medium energy should score well: {score}"
        );
    }

    // --- transition_score ---

    #[test]
    fn test_transition_score_perfect() {
        let a = CamelotKey {
            number: 8,
            letter: 'A',
        };
        let score = transition_score(Some(&a), Some(&a), Some(128.0), Some(128.0));
        // key=1.0*0.5 + bpm=1.0*0.3 + base=0.5*0.2 = 0.5 + 0.3 + 0.1 = 0.9
        assert!((score - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_transition_score_missing_data() {
        let score = transition_score(None, None, None, None);
        // key=0.5*0.5 + bpm=0.5*0.3 + base=0.5*0.2 = 0.25 + 0.15 + 0.1 = 0.5
        assert!((score - 0.5).abs() < 0.001);
    }
}
