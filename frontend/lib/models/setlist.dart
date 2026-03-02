import 'setlist_track.dart';

class ScoreBreakdown {
  final double keyCompatibility;
  final double bpmContinuity;
  final double energyArc;

  const ScoreBreakdown({
    required this.keyCompatibility,
    required this.bpmContinuity,
    required this.energyArc,
  });

  factory ScoreBreakdown.fromJson(Map<String, dynamic> json) {
    return ScoreBreakdown(
      keyCompatibility: (json['key_compatibility'] as num).toDouble(),
      bpmContinuity: (json['bpm_continuity'] as num).toDouble(),
      energyArc: (json['energy_arc'] as num).toDouble(),
    );
  }
}

class Setlist {
  final String id;
  final String prompt;
  final String model;
  final List<SetlistTrack> tracks;
  final String? notes;
  final double? harmonicFlowScore;
  final ScoreBreakdown? scoreBreakdown;
  final String? createdAt;

  const Setlist({
    required this.id,
    required this.prompt,
    required this.model,
    required this.tracks,
    this.notes,
    this.harmonicFlowScore,
    this.scoreBreakdown,
    this.createdAt,
  });

  factory Setlist.fromJson(Map<String, dynamic> json) {
    return Setlist(
      id: json['id'] as String,
      prompt: json['prompt'] as String,
      model: json['model'] as String,
      tracks: (json['tracks'] as List<dynamic>)
          .map((t) => SetlistTrack.fromJson(t as Map<String, dynamic>))
          .toList(),
      notes: json['notes'] as String?,
      harmonicFlowScore: (json['harmonic_flow_score'] as num?)?.toDouble(),
      scoreBreakdown: json['score_breakdown'] != null
          ? ScoreBreakdown.fromJson(
              json['score_breakdown'] as Map<String, dynamic>)
          : null,
      createdAt: json['created_at'] as String?,
    );
  }

  bool get isArranged => harmonicFlowScore != null;

  String get harmonicFlowScoreFormatted {
    if (harmonicFlowScore == null) return '--';
    return '${harmonicFlowScore!.round()}';
  }
}
