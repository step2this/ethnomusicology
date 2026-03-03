use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

// ---------------------------------------------------------------------------
// Energy Profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnergyProfile {
    WarmUp,   // 3→7: gentle build
    PeakTime, // 7→9→7: high energy plateau
    Journey,  // 3→9→4: full arc
    Steady,   // 6→6: consistent energy
}

impl fmt::Display for EnergyProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnergyProfile::WarmUp => write!(f, "warm-up"),
            EnergyProfile::PeakTime => write!(f, "peak-time"),
            EnergyProfile::Journey => write!(f, "journey"),
            EnergyProfile::Steady => write!(f, "steady"),
        }
    }
}

impl FromStr for EnergyProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "warm-up" => Ok(EnergyProfile::WarmUp),
            "peak-time" => Ok(EnergyProfile::PeakTime),
            "journey" => Ok(EnergyProfile::Journey),
            "steady" => Ok(EnergyProfile::Steady),
            _ => Err(format!("invalid energy profile: '{s}'")),
        }
    }
}

impl TryFrom<&str> for EnergyProfile {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
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
// Key Conversion Functions
// ---------------------------------------------------------------------------

/// Camelot codes for major keys indexed by pitch class (0=C through 11=B).
/// Each entry is (camelot_number, camelot_letter).
const MAJOR_CAMELOT: [(u8, char); 12] = [
    (8, 'B'),  // 0  C
    (3, 'B'),  // 1  C#
    (10, 'B'), // 2  D
    (5, 'B'),  // 3  Eb
    (12, 'B'), // 4  E
    (7, 'B'),  // 5  F
    (2, 'B'),  // 6  F#
    (9, 'B'),  // 7  G
    (4, 'B'),  // 8  Ab
    (11, 'B'), // 9  A
    (6, 'B'),  // 10 Bb
    (1, 'B'),  // 11 B
];

/// Camelot codes for minor keys indexed by pitch class (0=C through 11=B).
const MINOR_CAMELOT: [(u8, char); 12] = [
    (5, 'A'),  // 0  C
    (12, 'A'), // 1  C#
    (7, 'A'),  // 2  D
    (2, 'A'),  // 3  Eb
    (9, 'A'),  // 4  E
    (4, 'A'),  // 5  F
    (11, 'A'), // 6  F#
    (6, 'A'),  // 7  G
    (1, 'A'),  // 8  Ab
    (8, 'A'),  // 9  A
    (3, 'A'),  // 10 Bb
    (10, 'A'), // 11 B
];

/// Convert Spotify Audio Features key/mode to a CamelotKey.
///
/// `pitch_class`: 0-11 (C through B), from Spotify's `key` field.
/// `mode`: 0 = minor, 1 = major, from Spotify's `mode` field.
/// Returns `None` for out-of-range values.
pub fn from_spotify_key(pitch_class: i32, mode: i32) -> Option<CamelotKey> {
    if !(0..=11).contains(&pitch_class) {
        return None;
    }
    let idx = pitch_class as usize;
    let (number, letter) = match mode {
        1 => MAJOR_CAMELOT[idx],
        0 => MINOR_CAMELOT[idx],
        _ => return None,
    };
    Some(CamelotKey { number, letter })
}

/// Convert a musical note name and scale to a CamelotKey.
///
/// Handles essentia-style output (e.g., "C", "minor" → 5A).
/// - Supports sharps (`C#`) and flats (`Db`).
/// - Case-insensitive for both note and scale.
///
/// Returns `None` for unrecognized note names or scale types.
///
pub fn from_notation(note: &str, scale: &str) -> Option<CamelotKey> {
    let pitch_class = note_to_pitch_class(note)?;
    let mode = match scale.to_lowercase().as_str() {
        "major" => 1,
        "minor" => 0,
        _ => return None,
    };
    from_spotify_key(pitch_class, mode)
}

/// Parse a note name (e.g., "C#", "Db", "E") to a pitch class (0-11).
fn note_to_pitch_class(note: &str) -> Option<i32> {
    match note.to_lowercase().as_str() {
        "c" => Some(0),
        "c#" | "db" => Some(1),
        "d" => Some(2),
        "d#" | "eb" => Some(3),
        "e" => Some(4),
        "f" => Some(5),
        "f#" | "gb" => Some(6),
        "g" => Some(7),
        "g#" | "ab" => Some(8),
        "a" => Some(9),
        "a#" | "bb" => Some(10),
        "b" => Some(11),
        _ => None,
    }
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

/// Energy arc score parameterized by an energy profile.
///
/// Computes the ideal energy at each position based on the profile's target curve,
/// then scores how close the actual energy is to the ideal.
///
/// Score = 1.0 - (|actual_energy - ideal_energy| / 10.0), clamped to [0.0, 1.0].
pub fn energy_arc_score_with_profile(
    energy: i32,
    position: usize,
    total: usize,
    profile: EnergyProfile,
) -> f64 {
    if total <= 1 {
        return 1.0;
    }

    let fraction = position as f64 / (total - 1) as f64;

    let ideal_energy = match profile {
        EnergyProfile::WarmUp => {
            // Linear 3→7
            3.0 + fraction * 4.0
        }
        EnergyProfile::PeakTime => {
            // 7→9 at 40%, hold 9 at 40-60%, 9→7 from 60%
            if fraction < 0.4 {
                7.0 + (fraction / 0.4) * 2.0
            } else if fraction <= 0.6 {
                9.0
            } else {
                9.0 - ((fraction - 0.6) / 0.4) * 2.0
            }
        }
        EnergyProfile::Journey => {
            // 3→9 at 50%, 9→4 from 50%
            if fraction < 0.5 {
                3.0 + (fraction / 0.5) * 6.0
            } else {
                9.0 - ((fraction - 0.5) / 0.5) * 5.0
            }
        }
        EnergyProfile::Steady => {
            // Constant 6
            6.0
        }
    };

    let diff = (energy as f64 - ideal_energy).abs();
    (1.0 - diff / 10.0).clamp(0.0, 1.0)
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

    // --- from_spotify_key ---

    #[test]
    fn test_from_spotify_key_c_major() {
        let key = from_spotify_key(0, 1).unwrap();
        assert_eq!(key.number, 8);
        assert_eq!(key.letter, 'B');
    }

    #[test]
    fn test_from_spotify_key_a_minor() {
        let key = from_spotify_key(9, 0).unwrap();
        assert_eq!(key.number, 8);
        assert_eq!(key.letter, 'A');
    }

    #[test]
    fn test_from_spotify_key_all_major() {
        for pitch in 0..12 {
            assert!(
                from_spotify_key(pitch, 1).is_some(),
                "Failed for pitch {pitch} major"
            );
        }
    }

    #[test]
    fn test_from_spotify_key_all_minor() {
        for pitch in 0..12 {
            assert!(
                from_spotify_key(pitch, 0).is_some(),
                "Failed for pitch {pitch} minor"
            );
        }
    }

    #[test]
    fn test_from_spotify_key_invalid_pitch() {
        assert!(from_spotify_key(-1, 1).is_none());
        assert!(from_spotify_key(12, 0).is_none());
    }

    #[test]
    fn test_from_spotify_key_invalid_mode() {
        assert!(from_spotify_key(0, 2).is_none());
        assert!(from_spotify_key(0, -1).is_none());
    }

    // --- from_notation ---

    #[test]
    fn test_from_notation_c_minor() {
        let key = from_notation("C", "minor").unwrap();
        assert_eq!(key.number, 5);
        assert_eq!(key.letter, 'A');
    }

    #[test]
    fn test_from_notation_a_major() {
        let key = from_notation("A", "major").unwrap();
        assert_eq!(key.number, 11);
        assert_eq!(key.letter, 'B');
    }

    #[test]
    fn test_from_notation_sharp_flat() {
        let sharp = from_notation("C#", "minor").unwrap();
        let flat = from_notation("Db", "minor").unwrap();
        assert_eq!(sharp.number, flat.number);
        assert_eq!(sharp.letter, flat.letter);
    }

    #[test]
    fn test_from_notation_case_insensitive() {
        let key = from_notation("c", "Minor").unwrap();
        assert_eq!(key.number, 5);
        assert_eq!(key.letter, 'A');
    }

    #[test]
    fn test_from_notation_invalid() {
        assert!(from_notation("X", "minor").is_none());
        assert!(from_notation("C", "lydian").is_none());
    }

    // --- EnergyProfile ---

    #[test]
    fn test_energy_profile_from_str_valid() {
        assert_eq!(
            "warm-up".parse::<EnergyProfile>().unwrap(),
            EnergyProfile::WarmUp
        );
        assert_eq!(
            "peak-time".parse::<EnergyProfile>().unwrap(),
            EnergyProfile::PeakTime
        );
        assert_eq!(
            "journey".parse::<EnergyProfile>().unwrap(),
            EnergyProfile::Journey
        );
        assert_eq!(
            "steady".parse::<EnergyProfile>().unwrap(),
            EnergyProfile::Steady
        );
    }

    #[test]
    fn test_energy_profile_from_str_invalid() {
        assert!("invalid".parse::<EnergyProfile>().is_err());
        assert!("WARM-UP".parse::<EnergyProfile>().is_err());
        assert!("warmup".parse::<EnergyProfile>().is_err());
        assert!("".parse::<EnergyProfile>().is_err());
    }

    #[test]
    fn test_energy_profile_try_from_str() {
        assert_eq!(
            EnergyProfile::try_from("warm-up").unwrap(),
            EnergyProfile::WarmUp
        );
        assert!(EnergyProfile::try_from("invalid").is_err());
    }

    #[test]
    fn test_energy_profile_display() {
        assert_eq!(EnergyProfile::WarmUp.to_string(), "warm-up");
        assert_eq!(EnergyProfile::PeakTime.to_string(), "peak-time");
        assert_eq!(EnergyProfile::Journey.to_string(), "journey");
        assert_eq!(EnergyProfile::Steady.to_string(), "steady");
    }

    #[test]
    fn test_energy_profile_serde_roundtrip() {
        let profiles = [
            (EnergyProfile::WarmUp, "\"warm-up\""),
            (EnergyProfile::PeakTime, "\"peak-time\""),
            (EnergyProfile::Journey, "\"journey\""),
            (EnergyProfile::Steady, "\"steady\""),
        ];
        for (profile, expected_json) in &profiles {
            let serialized = serde_json::to_string(profile).unwrap();
            assert_eq!(&serialized, expected_json);
            let deserialized: EnergyProfile = serde_json::from_str(&serialized).unwrap();
            assert_eq!(&deserialized, profile);
        }
    }

    // --- energy_arc_score_with_profile ---

    #[test]
    fn test_warmup_profile_start_middle_end() {
        // WarmUp: 3→7 linear
        // Start (pos 0): ideal = 3.0
        let start = energy_arc_score_with_profile(3, 0, 10, EnergyProfile::WarmUp);
        assert!(
            start > 0.9,
            "WarmUp start with energy=3 should score high: {start}"
        );

        // Middle (pos 5): ideal = 3.0 + (5/9)*4.0 ≈ 5.22
        let mid = energy_arc_score_with_profile(5, 5, 10, EnergyProfile::WarmUp);
        assert!(
            mid > 0.9,
            "WarmUp middle with energy=5 should score high: {mid}"
        );

        // End (pos 9): ideal = 7.0
        let end = energy_arc_score_with_profile(7, 9, 10, EnergyProfile::WarmUp);
        assert!(
            end > 0.9,
            "WarmUp end with energy=7 should score high: {end}"
        );
    }

    #[test]
    fn test_peaktime_profile_start_middle_end() {
        // PeakTime: 7→9 at 40%, hold 9, 9→7 from 60%
        // Start (pos 0): ideal = 7.0
        let start = energy_arc_score_with_profile(7, 0, 10, EnergyProfile::PeakTime);
        assert!(
            start > 0.9,
            "PeakTime start with energy=7 should score high: {start}"
        );

        // Middle (pos 5, fraction=0.556): ideal = 9.0 (in plateau)
        let mid = energy_arc_score_with_profile(9, 5, 10, EnergyProfile::PeakTime);
        assert!(
            mid > 0.9,
            "PeakTime middle with energy=9 should score high: {mid}"
        );

        // End (pos 9): ideal = 7.0
        let end = energy_arc_score_with_profile(7, 9, 10, EnergyProfile::PeakTime);
        assert!(
            end > 0.9,
            "PeakTime end with energy=7 should score high: {end}"
        );
    }

    #[test]
    fn test_journey_profile_start_middle_end() {
        // Journey: 3→9 at 50%, 9→4 from 50%
        // Start (pos 0): ideal = 3.0
        let start = energy_arc_score_with_profile(3, 0, 10, EnergyProfile::Journey);
        assert!(
            start > 0.9,
            "Journey start with energy=3 should score high: {start}"
        );

        // Middle (pos 4, fraction=0.444): ideal = 3 + (0.444/0.5)*6 ≈ 8.33
        let mid = energy_arc_score_with_profile(8, 4, 10, EnergyProfile::Journey);
        assert!(
            mid > 0.9,
            "Journey middle with energy=8 should score high: {mid}"
        );

        // End (pos 9): ideal = 4.0
        let end = energy_arc_score_with_profile(4, 9, 10, EnergyProfile::Journey);
        assert!(
            end > 0.9,
            "Journey end with energy=4 should score high: {end}"
        );
    }

    #[test]
    fn test_steady_profile_constant() {
        // Steady: constant 6
        for pos in 0..10 {
            let score = energy_arc_score_with_profile(6, pos, 10, EnergyProfile::Steady);
            assert!(
                score > 0.9,
                "Steady with energy=6 at pos {pos} should score high: {score}"
            );
        }
    }

    #[test]
    fn test_energy_profile_score_penalizes_mismatch() {
        // WarmUp start should penalize high energy
        let score = energy_arc_score_with_profile(9, 0, 10, EnergyProfile::WarmUp);
        // ideal=3, actual=9, diff=6, score = 1.0 - 6/10 = 0.4
        assert!((score - 0.4).abs() < 0.01, "Expected ~0.4, got {score}");

        // Steady should penalize energy far from 6
        let score = energy_arc_score_with_profile(1, 5, 10, EnergyProfile::Steady);
        // ideal=6, actual=1, diff=5, score = 1.0 - 5/10 = 0.5
        assert!((score - 0.5).abs() < 0.01, "Expected ~0.5, got {score}");
    }

    #[test]
    fn test_energy_profile_single_track() {
        // Single track should always score 1.0 regardless of profile
        for profile in [
            EnergyProfile::WarmUp,
            EnergyProfile::PeakTime,
            EnergyProfile::Journey,
            EnergyProfile::Steady,
        ] {
            let score = energy_arc_score_with_profile(5, 0, 1, profile);
            assert_eq!(
                score, 1.0,
                "Single track should score 1.0 for {:?}",
                profile
            );
        }
    }

    #[test]
    fn test_energy_profile_two_tracks() {
        // WarmUp with 2 tracks: pos 0 → ideal 3, pos 1 → ideal 7
        let start = energy_arc_score_with_profile(3, 0, 2, EnergyProfile::WarmUp);
        let end = energy_arc_score_with_profile(7, 1, 2, EnergyProfile::WarmUp);
        assert!(start > 0.9, "WarmUp 2-track start: {start}");
        assert!(end > 0.9, "WarmUp 2-track end: {end}");
    }

    #[test]
    fn test_energy_profile_score_clamped() {
        // Score should never go below 0.0 even with extreme mismatch
        // energy=10, ideal=3.0 for WarmUp start, diff=7, score=0.3 (positive)
        let score = energy_arc_score_with_profile(10, 0, 10, EnergyProfile::WarmUp);
        assert!(score >= 0.0, "Score should be non-negative: {score}");
        assert!(score <= 1.0, "Score should not exceed 1.0: {score}");
    }
}
