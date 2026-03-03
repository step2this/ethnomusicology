import 'package:dio/dio.dart';

import '../models/setlist.dart';
import '../models/track_list_response.dart';

class ApiClient {
  static const _baseUrl = '/api';

  final Dio _dio;

  ApiClient({Dio? dio})
      : _dio = dio ??
            Dio(BaseOptions(
              baseUrl: _baseUrl,
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 30),
            ));

  Dio get dio => _dio;

  // -------------------------------------------------------------------------
  // Spotify OAuth
  // -------------------------------------------------------------------------

  Future<bool> checkSpotifyConnection(String userId) async {
    final response = await _dio.get(
      '/auth/spotify/status',
      options: Options(headers: {'X-User-Id': userId}),
    );
    return response.data['connected'] as bool? ?? false;
  }

  Future<String> getSpotifyAuthUrl(String userId) async {
    final response = await _dio.get(
      '/auth/spotify',
      options: Options(headers: {'X-User-Id': userId}),
    );
    return response.data['redirect_url'] as String;
  }

  // -------------------------------------------------------------------------
  // Spotify Import
  // -------------------------------------------------------------------------

  Future<Map<String, dynamic>> importSpotifyPlaylist(
      String playlistUrl) async {
    final response = await _dio.post(
      '/import/spotify',
      data: {'playlist_url': playlistUrl},
    );
    return response.data as Map<String, dynamic>;
  }

  Future<Map<String, dynamic>> getImportStatus(String importId) async {
    final response = await _dio.get('/import/$importId');
    return response.data as Map<String, dynamic>;
  }

  // -------------------------------------------------------------------------
  // Track Catalog
  // -------------------------------------------------------------------------

  Future<TrackListResponse> listTracks({
    int page = 1,
    int perPage = 25,
    String sort = 'date_added',
    String order = 'desc',
  }) async {
    final response = await _dio.get('/tracks', queryParameters: {
      'page': page,
      'per_page': perPage,
      'sort': sort,
      'order': order,
    });
    return TrackListResponse.fromJson(response.data as Map<String, dynamic>);
  }

  // -------------------------------------------------------------------------
  // Setlist Generation
  // -------------------------------------------------------------------------

  Future<Setlist> generateSetlist(
    String prompt, {
    int? trackCount,
    String? energyProfile,
    String? sourcePlaylistId,
    String? seedTracklist,
    bool? creativeMode,
    double? bpmMin,
    double? bpmMax,
  }) async {
    final data = <String, dynamic>{'prompt': prompt};
    if (trackCount != null) data['track_count'] = trackCount;
    if (energyProfile != null) data['energy_profile'] = energyProfile;
    if (sourcePlaylistId != null) {
      data['source_playlist_id'] = sourcePlaylistId;
    }
    if (seedTracklist != null) data['seed_tracklist'] = seedTracklist;
    if (creativeMode != null) data['creative_mode'] = creativeMode;
    if (bpmMin != null && bpmMax != null) {
      data['bpm_range'] = {'min': bpmMin, 'max': bpmMax};
    }
    final response = await _dio.post('/setlists/generate', data: data);
    return Setlist.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Setlist> arrangeSetlist(String id, {String? energyProfile}) async {
    final data = <String, dynamic>{};
    if (energyProfile != null) data['energy_profile'] = energyProfile;
    final response = await _dio.post(
      '/setlists/$id/arrange',
      data: data.isNotEmpty ? data : null,
    );
    return Setlist.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Setlist> getSetlist(String id) async {
    final response = await _dio.get('/setlists/$id');
    return Setlist.fromJson(response.data as Map<String, dynamic>);
  }

  // -------------------------------------------------------------------------
  // Audio Preview (Deezer)
  // -------------------------------------------------------------------------

  /// Search Deezer for a track preview URL. Returns the proxied preview URL or null.
  Future<String?> searchDeezerPreview(String title, String artist) async {
    try {
      final response = await _dio.get('/audio/deezer-search', queryParameters: {
        'q': '$artist $title',
        'limit': '1',
      });
      final data = response.data as Map<String, dynamic>;
      final results = data['data'] as List?;
      if (results == null || results.isEmpty) return null;
      final preview = results[0]['preview'] as String?;
      if (preview == null || preview.isEmpty) return null;
      // Return proxied URL to avoid CORS issues
      return '/api/audio/proxy?url=${Uri.encodeComponent(preview)}';
    } catch (e) {
      return null;
    }
  }
}
