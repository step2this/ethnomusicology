class Track {
  final String title;
  final String artist;
  final String album;
  final Duration duration;
  final String spotifyUri;
  final String? previewUrl;

  const Track({
    required this.title,
    required this.artist,
    required this.album,
    required this.duration,
    required this.spotifyUri,
    this.previewUrl,
  });

  factory Track.fromJson(Map<String, dynamic> json) {
    return Track(
      title: json['title'] as String,
      artist: json['artist'] as String,
      album: json['album'] as String,
      duration: Duration(milliseconds: json['duration_ms'] as int),
      spotifyUri: json['spotify_uri'] as String,
      previewUrl: json['preview_url'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'title': title,
      'artist': artist,
      'album': album,
      'duration_ms': duration.inMilliseconds,
      'spotify_uri': spotifyUri,
      'preview_url': previewUrl,
    };
  }
}
