class Crate {
  final String id;
  final String name;
  final int trackCount;
  final String createdAt;
  final String updatedAt;

  const Crate({
    required this.id,
    required this.name,
    required this.trackCount,
    required this.createdAt,
    required this.updatedAt,
  });

  factory Crate.fromJson(Map<String, dynamic> json) => Crate(
        id: json['id'] as String,
        name: json['name'] as String,
        trackCount: json['track_count'] as int,
        createdAt: json['created_at'] as String,
        updatedAt: json['updated_at'] as String,
      );
}

class CrateTrack {
  final String id;
  final int position;
  final String title;
  final String artist;
  final String? album;
  final double? bpm;
  final String? key;
  final String setlistId;
  final String addedAt;

  const CrateTrack({
    required this.id,
    required this.position,
    required this.title,
    required this.artist,
    this.album,
    this.bpm,
    this.key,
    required this.setlistId,
    required this.addedAt,
  });

  factory CrateTrack.fromJson(Map<String, dynamic> json) => CrateTrack(
        id: json['id'] as String,
        position: json['position'] as int,
        title: json['title'] as String,
        artist: json['artist'] as String,
        album: json['album'] as String?,
        bpm: (json['bpm'] as num?)?.toDouble(),
        key: json['key'] as String?,
        setlistId: json['setlist_id'] as String,
        addedAt: json['added_at'] as String,
      );
}

class CrateDetail {
  final String id;
  final String name;
  final List<CrateTrack> tracks;
  final String createdAt;
  final String updatedAt;

  const CrateDetail({
    required this.id,
    required this.name,
    required this.tracks,
    required this.createdAt,
    required this.updatedAt,
  });

  factory CrateDetail.fromJson(Map<String, dynamic> json) => CrateDetail(
        id: json['id'] as String,
        name: json['name'] as String,
        tracks: (json['tracks'] as List<dynamic>)
            .map((t) => CrateTrack.fromJson(t as Map<String, dynamic>))
            .toList(),
        createdAt: json['created_at'] as String,
        updatedAt: json['updated_at'] as String,
      );
}
