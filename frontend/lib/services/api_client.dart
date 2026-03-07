import 'package:dio/dio.dart';

import '../models/crate.dart';
import '../models/purchase_link.dart';
import '../models/refinement.dart';
import '../models/setlist.dart';
import '../models/track_list_response.dart';
import '../providers/setlist_library_provider.dart';

class ApiClient {
  static const _baseUrl = '/api';

  final Dio _dio;

  ApiClient({Dio? dio})
      : _dio = dio ??
            Dio(BaseOptions(
              baseUrl: _baseUrl,
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 120),
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
    bool? verify,
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
    if (verify != null) data['verify'] = verify;
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

  Future<List<SetlistSummary>> listSetlists() async {
    final response = await _dio.get('/setlists');
    final list = (response.data['setlists'] as List<dynamic>?) ?? [];
    return list
        .map((e) => SetlistSummary.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<void> deleteSetlist(String id) async {
    await _dio.delete('/setlists/$id');
  }

  Future<Setlist> updateSetlist(String id, {String? name}) async {
    final response = await _dio.patch(
      '/setlists/$id',
      data: {'name': name},
    );
    return Setlist.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Setlist> duplicateSetlist(String id) async {
    final response = await _dio.post('/setlists/$id/duplicate');
    return Setlist.fromJson(response.data as Map<String, dynamic>);
  }

  // -------------------------------------------------------------------------
  // Crates
  // -------------------------------------------------------------------------

  Future<List<Crate>> listCrates() async {
    final response = await _dio.get('/crates');
    final list = (response.data['crates'] as List<dynamic>?) ?? [];
    return list
        .map((e) => Crate.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<Crate> createCrate(String name) async {
    final response = await _dio.post('/crates', data: {'name': name});
    return Crate.fromJson(response.data as Map<String, dynamic>);
  }

  Future<CrateDetail> getCrate(String id) async {
    final response = await _dio.get('/crates/$id');
    return CrateDetail.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteCrate(String id) async {
    await _dio.delete('/crates/$id');
  }

  Future<void> addSetlistToCrate(String crateId, String setlistId) async {
    await _dio.post('/crates/$crateId/setlists/$setlistId');
  }

  Future<void> removeCrateTrack(String crateId, String trackId) async {
    await _dio.delete('/crates/$crateId/tracks/$trackId');
  }

  // -------------------------------------------------------------------------
  // Purchase Links
  // -------------------------------------------------------------------------

  Future<List<PurchaseLink>> getPurchaseLinks({
    required String title,
    required String artist,
  }) async {
    final response = await _dio.get('/purchase-links', queryParameters: {
      'title': title,
      'artist': artist,
    });
    final links = (response.data['links'] as List)
        .map((e) => PurchaseLink.fromJson(e as Map<String, dynamic>))
        .toList();
    return links;
  }

  // -------------------------------------------------------------------------
  // Audio Preview (unified: Deezer → iTunes fallback)
  // -------------------------------------------------------------------------

  /// Unified preview search: backend tries Deezer then iTunes.
  /// Returns a PreviewSearchResult with source, preview URL, and metadata.
  Future<PreviewSearchResult> searchPreview(String title, String artist) async {
    try {
      final response = await _dio.get('/audio/search', queryParameters: {
        'title': title,
        'artist': artist,
      });
      final data = response.data as Map<String, dynamic>;
      return PreviewSearchResult(
        source: data['source'] as String?,
        previewUrl: data['preview_url'] as String?,
        externalUrl: data['external_url'] as String?,
        searchQueries: (data['search_queries'] as List?)
                ?.map((e) => e as String)
                .toList() ??
            [],
        uploaderName: data['uploader_name'] as String?,
      );
    } catch (e) {
      return const PreviewSearchResult(
        source: null,
        previewUrl: null,
        externalUrl: null,
        searchQueries: [],
        uploaderName: null,
      );
    }
  }

  // -------------------------------------------------------------------------
  // Setlist Refinement
  // -------------------------------------------------------------------------

  Future<RefinementResponse> refineSetlist(String setlistId, String message) async {
    final response = await _dio.post(
      '/setlists/$setlistId/refine',
      data: {'message': message},
    );
    return RefinementResponse.fromJson(response.data as Map<String, dynamic>);
  }

  Future<RefinementResponse> revertSetlist(String setlistId, int versionNumber) async {
    final response = await _dio.post('/setlists/$setlistId/revert/$versionNumber');
    return RefinementResponse.fromJson(response.data as Map<String, dynamic>);
  }

  Future<HistoryResponse> getSetlistHistory(String setlistId) async {
    final response = await _dio.get('/setlists/$setlistId/history');
    return HistoryResponse.fromJson(response.data as Map<String, dynamic>);
  }
}

class PreviewSearchResult {
  final String? source;
  final String? previewUrl;
  final String? externalUrl;
  final List<String> searchQueries;
  final String? uploaderName;

  const PreviewSearchResult({
    required this.source,
    required this.previewUrl,
    required this.externalUrl,
    required this.searchQueries,
    this.uploaderName,
  });
}
