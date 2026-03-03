use crate::services::camelot::{self, CamelotKey, EnergyProfile, ScoreBreakdown};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub struct ArrangementTrack {
    pub index: usize,
    pub camelot: Option<CamelotKey>,
    pub bpm: Option<f64>,
    pub energy: Option<i32>,
}

pub struct ArrangementResult {
    pub ordered_indices: Vec<usize>,
    pub transition_scores: Vec<f64>,
    pub harmonic_flow_score: f64,
    pub score_breakdown: ScoreBreakdown,
}

// ---------------------------------------------------------------------------
// Arrangement Algorithm: Greedy Nearest-Neighbor + 2-opt
// ---------------------------------------------------------------------------

/// Arrange tracks for optimal harmonic flow using greedy nearest-neighbor + 2-opt optimization.
///
/// If `energy_profile` is `Some`, uses the profiled energy arc scoring.
/// If `None`, uses the default energy arc scoring for backward compatibility.
pub fn arrange_tracks(
    tracks: &[ArrangementTrack],
    energy_profile: Option<EnergyProfile>,
) -> ArrangementResult {
    if tracks.is_empty() {
        return ArrangementResult {
            ordered_indices: vec![],
            transition_scores: vec![],
            harmonic_flow_score: 0.0,
            score_breakdown: ScoreBreakdown {
                key_compatibility: 0.0,
                bpm_continuity: 0.0,
                energy_arc: 0.0,
            },
        };
    }

    if tracks.len() == 1 {
        return ArrangementResult {
            ordered_indices: vec![tracks[0].index],
            transition_scores: vec![],
            harmonic_flow_score: 100.0,
            score_breakdown: ScoreBreakdown {
                key_compatibility: 100.0,
                bpm_continuity: 100.0,
                energy_arc: 100.0,
            },
        };
    }

    // Step 1: Greedy start — pick the track with lowest energy as opener
    let mut order: Vec<usize> = Vec::with_capacity(tracks.len());
    let mut visited = vec![false; tracks.len()];

    let start_idx = tracks
        .iter()
        .enumerate()
        .min_by_key(|(_, t)| t.energy.unwrap_or(5))
        .map(|(i, _)| i)
        .unwrap_or(0);

    order.push(start_idx);
    visited[start_idx] = true;

    // Step 2: Greedy nearest-neighbor by transition score
    for _ in 1..tracks.len() {
        let last = order[order.len() - 1];
        let mut best_idx = None;
        let mut best_score = -1.0;

        for (i, track) in tracks.iter().enumerate() {
            if visited[i] {
                continue;
            }
            let score = camelot::transition_score(
                tracks[last].camelot.as_ref(),
                track.camelot.as_ref(),
                tracks[last].bpm,
                track.bpm,
            );
            if score > best_score {
                best_score = score;
                best_idx = Some(i);
            }
        }

        if let Some(idx) = best_idx {
            order.push(idx);
            visited[idx] = true;
        }
    }

    // Step 3: 2-opt improvement
    let max_iterations = 100;
    for _ in 0..max_iterations {
        let mut improved = false;
        for i in 1..order.len().saturating_sub(1) {
            for j in (i + 1)..order.len() {
                let current_cost = segment_cost(tracks, &order, i, j);
                // Reverse segment [i..=j]
                order[i..=j].reverse();
                let new_cost = segment_cost(tracks, &order, i, j);
                if new_cost > current_cost {
                    improved = true; // keep the reversal
                } else {
                    order[i..=j].reverse(); // revert
                }
            }
        }
        if !improved {
            break;
        }
    }

    // Step 4: Compute scores
    let total = tracks.len();
    let mut t_scores = Vec::with_capacity(total.saturating_sub(1));
    let mut key_scores = Vec::new();
    let mut bpm_scores = Vec::new();

    for i in 0..total.saturating_sub(1) {
        let a = &tracks[order[i]];
        let b = &tracks[order[i + 1]];
        let ts = camelot::transition_score(a.camelot.as_ref(), b.camelot.as_ref(), a.bpm, b.bpm);
        t_scores.push(ts);

        // Individual component scores
        match (a.camelot.as_ref(), b.camelot.as_ref()) {
            (Some(ka), Some(kb)) => key_scores.push(camelot::camelot_score(ka, kb)),
            _ => key_scores.push(0.5),
        }
        match (a.bpm, b.bpm) {
            (Some(ba), Some(bb)) => bpm_scores.push(camelot::bpm_score(ba, bb)),
            _ => bpm_scores.push(0.5),
        }
    }

    let mut energy_scores = Vec::new();
    for (pos, &idx) in order.iter().enumerate() {
        if let Some(e) = tracks[idx].energy {
            let score = match energy_profile {
                Some(profile) => camelot::energy_arc_score_with_profile(e, pos, total, profile),
                None => camelot::energy_arc_score(e, pos, total),
            };
            energy_scores.push(score);
        } else {
            energy_scores.push(0.5);
        }
    }

    let avg = |v: &[f64]| -> f64 {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };

    let harmonic_flow = avg(&t_scores) * 100.0;
    let score_breakdown = ScoreBreakdown {
        key_compatibility: avg(&key_scores) * 100.0,
        bpm_continuity: avg(&bpm_scores) * 100.0,
        energy_arc: avg(&energy_scores) * 100.0,
    };

    let ordered_indices: Vec<usize> = order.iter().map(|&i| tracks[i].index).collect();

    ArrangementResult {
        ordered_indices,
        transition_scores: t_scores,
        harmonic_flow_score: harmonic_flow,
        score_breakdown,
    }
}

/// Cost of the segment at boundaries i and j for 2-opt evaluation.
fn segment_cost(tracks: &[ArrangementTrack], order: &[usize], i: usize, j: usize) -> f64 {
    let mut cost = 0.0;

    // Edge before i
    if i > 0 {
        let a = &tracks[order[i - 1]];
        let b = &tracks[order[i]];
        cost += camelot::transition_score(a.camelot.as_ref(), b.camelot.as_ref(), a.bpm, b.bpm);
    }

    // Edge after j
    if j + 1 < order.len() {
        let a = &tracks[order[j]];
        let b = &tracks[order[j + 1]];
        cost += camelot::transition_score(a.camelot.as_ref(), b.camelot.as_ref(), a.bpm, b.bpm);
    }

    cost
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::camelot::parse_camelot;

    fn make_track(
        index: usize,
        camelot: Option<&str>,
        bpm: Option<f64>,
        energy: Option<i32>,
    ) -> ArrangementTrack {
        ArrangementTrack {
            index,
            camelot: camelot.and_then(parse_camelot),
            bpm,
            energy,
        }
    }

    #[test]
    fn test_empty_input() {
        let result = arrange_tracks(&[], None);
        assert!(result.ordered_indices.is_empty());
        assert!(result.transition_scores.is_empty());
    }

    #[test]
    fn test_single_track() {
        let tracks = vec![make_track(0, Some("8A"), Some(128.0), Some(5))];
        let result = arrange_tracks(&tracks, None);
        assert_eq!(result.ordered_indices, vec![0]);
        assert!(result.transition_scores.is_empty());
        assert_eq!(result.harmonic_flow_score, 100.0);
    }

    #[test]
    fn test_two_tracks() {
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(5)),
            make_track(1, Some("9A"), Some(130.0), Some(7)),
        ];
        let result = arrange_tracks(&tracks, None);
        assert_eq!(result.ordered_indices.len(), 2);
        assert_eq!(result.transition_scores.len(), 1);
        assert!(result.harmonic_flow_score > 0.0);
    }

    #[test]
    fn test_camelot_compatible_grouping() {
        // Tracks with compatible keys should end up adjacent
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(3)),
            make_track(1, Some("1A"), Some(140.0), Some(5)), // distant from 8A
            make_track(2, Some("9A"), Some(130.0), Some(6)), // adjacent to 8A
            make_track(3, Some("7A"), Some(126.0), Some(4)), // adjacent to 8A
        ];
        let result = arrange_tracks(&tracks, None);
        assert_eq!(result.ordered_indices.len(), 4);

        // The distant key (1A) should not be between compatible keys.
        // Compatible keys (8A, 9A, 7A) should cluster together, pushing the
        // distant key (1A) to one end of the arrangement.
        let indices = &result.ordered_indices;
        let pos_8a = indices.iter().position(|&i| i == 0).unwrap();
        let pos_9a = indices.iter().position(|&i| i == 2).unwrap();
        let pos_7a = indices.iter().position(|&i| i == 3).unwrap();
        let pos_1a = indices.iter().position(|&i| i == 1).unwrap();

        // The three compatible keys (7A, 8A, 9A) should span at most 2
        // positions (i.e., be adjacent). The spread is the difference between
        // max and min positions within the compatible group.
        let compatible_positions = [pos_8a, pos_9a, pos_7a];
        let min_pos = *compatible_positions.iter().min().unwrap();
        let max_pos = *compatible_positions.iter().max().unwrap();
        let compatible_span = max_pos - min_pos;

        assert!(
            compatible_span <= 2,
            "Compatible keys (7A, 8A, 9A) should be clustered within 3 adjacent positions, \
             but span was {compatible_span} (positions: 8A={pos_8a}, 9A={pos_9a}, 7A={pos_7a})"
        );

        // The distant key (1A) should be at one end (first or last position)
        assert!(
            pos_1a == 0 || pos_1a == indices.len() - 1,
            "Distant key 1A should be at an edge of the arrangement, but was at position {pos_1a}"
        );

        // Additionally verify the arrangement produces a reasonable harmonic flow score
        assert!(
            result.harmonic_flow_score > 50.0,
            "Arrangement with mostly compatible keys should score above 50, got {}",
            result.harmonic_flow_score
        );
    }

    #[test]
    fn test_deterministic() {
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(3)),
            make_track(1, Some("9A"), Some(130.0), Some(6)),
            make_track(2, Some("10A"), Some(132.0), Some(8)),
            make_track(3, Some("7A"), Some(126.0), Some(4)),
        ];

        let first_result = arrange_tracks(&tracks, None);
        for _ in 0..10 {
            let result = arrange_tracks(&tracks, None);
            assert_eq!(
                result.ordered_indices, first_result.ordered_indices,
                "Arrangement should be deterministic"
            );
        }
    }

    #[test]
    fn test_preserves_count() {
        let tracks: Vec<_> = (0..8)
            .map(|i| make_track(i, Some("8A"), Some(128.0), Some(5)))
            .collect();
        let result = arrange_tracks(&tracks, None);
        assert_eq!(result.ordered_indices.len(), 8);
        // All indices present
        let mut sorted = result.ordered_indices.clone();
        sorted.sort();
        assert_eq!(sorted, (0..8).collect::<Vec<_>>());
    }

    #[test]
    fn test_all_missing_data() {
        let tracks = vec![
            make_track(0, None, None, None),
            make_track(1, None, None, None),
            make_track(2, None, None, None),
        ];
        let result = arrange_tracks(&tracks, None);
        assert_eq!(result.ordered_indices.len(), 3);
        // Should still produce a result with neutral scores
        assert!(result.harmonic_flow_score >= 0.0);
    }

    #[test]
    fn test_mixed_data() {
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(5)),
            make_track(1, None, None, None),
            make_track(2, Some("9A"), Some(130.0), None),
        ];
        let result = arrange_tracks(&tracks, None);
        assert_eq!(result.ordered_indices.len(), 3);
    }

    #[test]
    fn test_energy_arc_respected() {
        // Low energy track should start, high energy in middle
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(9)), // high energy
            make_track(1, Some("8A"), Some(128.0), Some(2)), // low energy
            make_track(2, Some("8A"), Some(128.0), Some(5)), // medium energy
        ];
        let result = arrange_tracks(&tracks, None);
        // First track should be the lowest energy one (index 1, energy=2)
        assert_eq!(result.ordered_indices[0], 1, "Lowest energy should open");
    }

    #[test]
    fn test_two_opt_improvement() {
        // Create a scenario where 2-opt should help:
        // Greedy might pick A→C→B→D but optimal is A→B→C→D
        let tracks = vec![
            make_track(0, Some("1A"), Some(120.0), Some(2)),
            make_track(1, Some("2A"), Some(122.0), Some(4)),
            make_track(2, Some("3A"), Some(124.0), Some(6)),
            make_track(3, Some("4A"), Some(126.0), Some(8)),
        ];
        let result = arrange_tracks(&tracks, None);
        // With ascending keys, the score should be high
        assert!(
            result.harmonic_flow_score > 50.0,
            "Score: {}",
            result.harmonic_flow_score
        );
    }

    #[test]
    fn test_performance_20_tracks() {
        let tracks: Vec<_> = (0..20)
            .map(|i| {
                let key_num = (i % 12) + 1;
                let letter = if i % 2 == 0 { "A" } else { "B" };
                let key = format!("{key_num}{letter}");
                make_track(
                    i,
                    Some(&key),
                    Some(120.0 + i as f64),
                    Some((i % 10 + 1) as i32),
                )
            })
            .collect();

        let start = std::time::Instant::now();
        let result = arrange_tracks(&tracks, None);
        let elapsed = start.elapsed();

        assert_eq!(result.ordered_indices.len(), 20);
        assert!(
            elapsed.as_millis() < 500,
            "Arrangement took {}ms, should be <500ms",
            elapsed.as_millis()
        );
    }

    // --- arrange_tracks with EnergyProfile ---

    #[test]
    fn test_arrange_with_none_profile_matches_default() {
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(3)),
            make_track(1, Some("9A"), Some(130.0), Some(6)),
            make_track(2, Some("10A"), Some(132.0), Some(8)),
            make_track(3, Some("7A"), Some(126.0), Some(4)),
        ];

        let result_none = arrange_tracks(&tracks, None);
        let result_none2 = arrange_tracks(&tracks, None);

        assert_eq!(
            result_none.ordered_indices, result_none2.ordered_indices,
            "None profile should produce identical results"
        );
    }

    #[test]
    fn test_arrange_with_different_profiles_produce_different_scores() {
        // Energy profiles affect the energy_arc score component, not the ordering
        // (ordering is driven by key+BPM transition scores). Different profiles
        // should produce different energy arc scores on the same arrangement.
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(2)), // low energy
            make_track(1, Some("8A"), Some(128.0), Some(9)), // high energy
            make_track(2, Some("8A"), Some(128.0), Some(6)), // medium energy
            make_track(3, Some("8A"), Some(128.0), Some(4)), // low-medium
            make_track(4, Some("8A"), Some(128.0), Some(7)), // medium-high
            make_track(5, Some("8A"), Some(128.0), Some(3)), // low
            make_track(6, Some("8A"), Some(128.0), Some(8)), // high
            make_track(7, Some("8A"), Some(128.0), Some(5)), // medium
        ];

        let result_warmup = arrange_tracks(&tracks, Some(EnergyProfile::WarmUp));
        let result_peaktime = arrange_tracks(&tracks, Some(EnergyProfile::PeakTime));
        let result_steady = arrange_tracks(&tracks, Some(EnergyProfile::Steady));

        // Different profiles should produce different energy arc scores
        let warmup_energy = result_warmup.score_breakdown.energy_arc;
        let peaktime_energy = result_peaktime.score_breakdown.energy_arc;
        let steady_energy = result_steady.score_breakdown.energy_arc;

        let all_same = (warmup_energy - peaktime_energy).abs() < 0.01
            && (peaktime_energy - steady_energy).abs() < 0.01;
        assert!(
            !all_same,
            "Different energy profiles should produce different energy arc scores: \
             WarmUp={warmup_energy:.2}, PeakTime={peaktime_energy:.2}, Steady={steady_energy:.2}"
        );
    }

    #[test]
    fn test_arrange_with_warmup_prefers_ascending_energy() {
        // WarmUp: 3→7, so lowest energy should be first
        let tracks = vec![
            make_track(0, Some("8A"), Some(128.0), Some(7)),
            make_track(1, Some("8A"), Some(128.0), Some(3)),
            make_track(2, Some("8A"), Some(128.0), Some(5)),
        ];

        let result = arrange_tracks(&tracks, Some(EnergyProfile::WarmUp));
        // First track should be the lowest energy (3)
        assert_eq!(
            result.ordered_indices[0], 1,
            "WarmUp should start with lowest energy track"
        );
    }

    #[test]
    fn test_arrange_with_profile_preserves_count() {
        let tracks: Vec<_> = (0..6)
            .map(|i| make_track(i, Some("8A"), Some(128.0), Some((i + 1) as i32)))
            .collect();

        for profile in [
            EnergyProfile::WarmUp,
            EnergyProfile::PeakTime,
            EnergyProfile::Journey,
            EnergyProfile::Steady,
        ] {
            let result = arrange_tracks(&tracks, Some(profile));
            assert_eq!(
                result.ordered_indices.len(),
                6,
                "Profile {:?} should preserve track count",
                profile
            );
            let mut sorted = result.ordered_indices.clone();
            sorted.sort();
            assert_eq!(sorted, (0..6).collect::<Vec<_>>());
        }
    }

    #[test]
    fn test_arrange_with_profile_empty_tracks() {
        let result = arrange_tracks(&[], Some(EnergyProfile::Journey));
        assert!(result.ordered_indices.is_empty());
    }

    #[test]
    fn test_arrange_with_profile_single_track() {
        let tracks = vec![make_track(0, Some("8A"), Some(128.0), Some(5))];
        let result = arrange_tracks(&tracks, Some(EnergyProfile::PeakTime));
        assert_eq!(result.ordered_indices, vec![0]);
        assert_eq!(result.harmonic_flow_score, 100.0);
    }
}
