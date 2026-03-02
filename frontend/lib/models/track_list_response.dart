import 'track.dart';

class TrackListResponse {
  final List<Track> data;
  final int page;
  final int perPage;
  final int total;
  final int totalPages;

  const TrackListResponse({
    required this.data,
    required this.page,
    required this.perPage,
    required this.total,
    required this.totalPages,
  });

  factory TrackListResponse.fromJson(Map<String, dynamic> json) {
    return TrackListResponse(
      data: (json['data'] as List<dynamic>)
          .map((item) => Track.fromJson(item as Map<String, dynamic>))
          .toList(),
      page: json['page'] as int,
      perPage: json['per_page'] as int,
      total: json['total'] as int,
      totalPages: json['total_pages'] as int,
    );
  }
}
