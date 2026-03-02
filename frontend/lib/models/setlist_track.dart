class SetlistTrack {
  final int position;
  final String title;
  final String artist;
  final double? bpm;
  final String? key;
  final String? camelot;
  final int? energy;
  final String? transitionNote;
  final double? transitionScore;
  final int originalPosition;
  final String source;
  final String? trackId;

  const SetlistTrack({
    required this.position,
    required this.title,
    required this.artist,
    this.bpm,
    this.key,
    this.camelot,
    this.energy,
    this.transitionNote,
    this.transitionScore,
    required this.originalPosition,
    required this.source,
    this.trackId,
  });

  factory SetlistTrack.fromJson(Map<String, dynamic> json) {
    return SetlistTrack(
      position: json['position'] as int,
      title: json['title'] as String,
      artist: json['artist'] as String,
      bpm: (json['bpm'] as num?)?.toDouble(),
      key: json['key'] as String?,
      camelot: json['camelot'] as String?,
      energy: (json['energy'] as num?)?.toInt(),
      transitionNote: json['transition_note'] as String?,
      transitionScore: (json['transition_score'] as num?)?.toDouble(),
      originalPosition: json['original_position'] as int,
      source: json['source'] as String? ?? 'suggestion',
      trackId: json['track_id'] as String?,
    );
  }

  String get bpmFormatted => bpm != null ? bpm!.round().toString() : '--';
  String get camelotFormatted => camelot ?? '--';
  String get energyFormatted => energy != null ? '$energy' : '--';

  bool get isCatalogTrack => source == 'catalog';
  bool get hasTransitionScore => transitionScore != null;

  String get transitionScoreFormatted {
    if (transitionScore == null) return '--';
    return '${(transitionScore! * 100).round()}%';
  }
}
