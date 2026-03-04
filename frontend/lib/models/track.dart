class Track {
  final String id;
  final String title;
  final String artist;
  final String? album;
  final int? durationMs;
  final double? bpm;
  final String? key;
  final double? energy;
  final String source;
  final String? sourceId;
  final String? previewUrl;
  final String? albumArtUrl;
  final DateTime dateAdded;

  const Track({
    required this.id,
    required this.title,
    required this.artist,
    this.album,
    this.durationMs,
    this.bpm,
    this.key,
    this.energy,
    required this.source,
    this.sourceId,
    this.previewUrl,
    this.albumArtUrl,
    required this.dateAdded,
  });

  factory Track.fromJson(Map<String, dynamic> json) {
    return Track(
      id: json['id'] as String,
      title: json['title'] as String,
      artist: json['artist'] as String,
      album: json['album'] as String?,
      durationMs: json['duration_ms'] as int?,
      bpm: (json['bpm'] as num?)?.toDouble(),
      key: json['key'] as String?,
      energy: (json['energy'] as num?)?.toDouble(),
      source: json['source'] as String? ?? 'spotify',
      sourceId: json['source_id'] as String?,
      previewUrl: json['preview_url'] as String?,
      albumArtUrl: json['album_art_url'] as String?,
      dateAdded: DateTime.parse(json['date_added'] as String),
    );
  }

  /// Format duration as "m:ss" (e.g., "3:45"). Returns "--" if null.
  String get durationFormatted {
    if (durationMs == null) return '--';
    final d = Duration(milliseconds: durationMs!);
    final minutes = d.inMinutes;
    final seconds = d.inSeconds.remainder(60);
    return '$minutes:${seconds.toString().padLeft(2, '0')}';
  }

  /// Format BPM as integer string (e.g., "128"). Returns "--" if null.
  String get bpmFormatted => bpm != null ? bpm!.round().toString() : '--';

  /// Format key. Returns "--" if null.
  String get keyFormatted => key ?? '--';
}
