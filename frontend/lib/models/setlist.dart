import 'setlist_track.dart';

class BpmWarning {
  final int fromPosition;
  final int toPosition;
  final double bpmDelta;

  const BpmWarning({
    required this.fromPosition,
    required this.toPosition,
    required this.bpmDelta,
  });

  factory BpmWarning.fromJson(Map<String, dynamic> json) => BpmWarning(
        fromPosition: json['from_position'] as int,
        toPosition: json['to_position'] as int,
        bpmDelta: (json['bpm_delta'] as num).toDouble(),
      );
}

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
  final String? energyProfile;
  final double? catalogPercentage;
  final String? catalogWarning;
  final List<BpmWarning> bpmWarnings;

  const Setlist({
    required this.id,
    required this.prompt,
    required this.model,
    required this.tracks,
    this.notes,
    this.harmonicFlowScore,
    this.scoreBreakdown,
    this.createdAt,
    this.energyProfile,
    this.catalogPercentage,
    this.catalogWarning,
    this.bpmWarnings = const [],
  });

  Setlist copyWith({
    String? id,
    String? prompt,
    String? model,
    List<SetlistTrack>? tracks,
    String? Function()? notes,
    double? Function()? harmonicFlowScore,
    ScoreBreakdown? Function()? scoreBreakdown,
    String? Function()? createdAt,
    String? Function()? energyProfile,
    double? Function()? catalogPercentage,
    String? Function()? catalogWarning,
    List<BpmWarning>? bpmWarnings,
  }) {
    return Setlist(
      id: id ?? this.id,
      prompt: prompt ?? this.prompt,
      model: model ?? this.model,
      tracks: tracks ?? this.tracks,
      notes: notes != null ? notes() : this.notes,
      harmonicFlowScore: harmonicFlowScore != null ? harmonicFlowScore() : this.harmonicFlowScore,
      scoreBreakdown: scoreBreakdown != null ? scoreBreakdown() : this.scoreBreakdown,
      createdAt: createdAt != null ? createdAt() : this.createdAt,
      energyProfile: energyProfile != null ? energyProfile() : this.energyProfile,
      catalogPercentage: catalogPercentage != null ? catalogPercentage() : this.catalogPercentage,
      catalogWarning: catalogWarning != null ? catalogWarning() : this.catalogWarning,
      bpmWarnings: bpmWarnings ?? this.bpmWarnings,
    );
  }

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
      energyProfile: json['energy_profile'] as String?,
      catalogPercentage:
          (json['catalog_percentage'] as num?)?.toDouble(),
      catalogWarning: json['catalog_warning'] as String?,
      bpmWarnings: (json['bpm_warnings'] as List<dynamic>?)
              ?.map(
                  (w) => BpmWarning.fromJson(w as Map<String, dynamic>))
              .toList() ??
          const [],
    );
  }

  bool get isArranged => harmonicFlowScore != null;

  String get harmonicFlowScoreFormatted {
    if (harmonicFlowScore == null) return '--';
    return '${harmonicFlowScore!.round()}';
  }
}
