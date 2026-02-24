import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/api_client.dart';

// ---------------------------------------------------------------------------
// State models
// ---------------------------------------------------------------------------

enum SpotifyConnectionStatus { disconnected, connecting, connected, error }

class SpotifyConnectionState {
  final SpotifyConnectionStatus status;
  final String? errorMessage;

  const SpotifyConnectionState({
    this.status = SpotifyConnectionStatus.disconnected,
    this.errorMessage,
  });

  SpotifyConnectionState copyWith({
    SpotifyConnectionStatus? status,
    String? errorMessage,
  }) {
    return SpotifyConnectionState(
      status: status ?? this.status,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

enum ImportStatus { idle, validating, importing, completed, error }

class ImportProgress {
  final int total;
  final int inserted;
  final int updated;
  final int failed;

  const ImportProgress({
    this.total = 0,
    this.inserted = 0,
    this.updated = 0,
    this.failed = 0,
  });
}

class SpotifyImportState {
  final ImportStatus status;
  final String? importId;
  final ImportProgress progress;
  final String? errorMessage;

  const SpotifyImportState({
    this.status = ImportStatus.idle,
    this.importId,
    this.progress = const ImportProgress(),
    this.errorMessage,
  });

  SpotifyImportState copyWith({
    ImportStatus? status,
    String? importId,
    ImportProgress? progress,
    String? errorMessage,
  }) {
    return SpotifyImportState(
      status: status ?? this.status,
      importId: importId ?? this.importId,
      progress: progress ?? this.progress,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

// ---------------------------------------------------------------------------
// Connection provider
// ---------------------------------------------------------------------------

class SpotifyConnectionNotifier extends StateNotifier<SpotifyConnectionState> {
  final ApiClient _api;

  SpotifyConnectionNotifier(this._api)
      : super(const SpotifyConnectionState());

  Future<void> checkConnection(String userId) async {
    state = state.copyWith(status: SpotifyConnectionStatus.connecting);
    try {
      final connected = await _api.checkSpotifyConnection(userId);
      state = state.copyWith(
        status: connected
            ? SpotifyConnectionStatus.connected
            : SpotifyConnectionStatus.disconnected,
      );
    } catch (e) {
      state = state.copyWith(
        status: SpotifyConnectionStatus.error,
        errorMessage: e.toString(),
      );
    }
  }

  Future<String?> getAuthorizationUrl(String userId) async {
    try {
      return await _api.getSpotifyAuthUrl(userId);
    } catch (e) {
      state = state.copyWith(
        status: SpotifyConnectionStatus.error,
        errorMessage: e.toString(),
      );
      return null;
    }
  }

  void setConnected() {
    state = state.copyWith(status: SpotifyConnectionStatus.connected);
  }
}

final spotifyConnectionProvider =
    StateNotifierProvider<SpotifyConnectionNotifier, SpotifyConnectionState>(
  (ref) => SpotifyConnectionNotifier(ref.read(apiClientProvider)),
);

// ---------------------------------------------------------------------------
// Import provider
// ---------------------------------------------------------------------------

class SpotifyImportNotifier extends StateNotifier<SpotifyImportState> {
  final ApiClient _api;

  SpotifyImportNotifier(this._api) : super(const SpotifyImportState());

  Future<void> importPlaylist(String playlistUrl) async {
    state = state.copyWith(status: ImportStatus.validating);

    try {
      state = state.copyWith(status: ImportStatus.importing);
      final result = await _api.importSpotifyPlaylist(playlistUrl);

      state = state.copyWith(
        status: ImportStatus.completed,
        importId: result['import_id'] as String?,
        progress: ImportProgress(
          total: result['total'] as int? ?? 0,
          inserted: result['inserted'] as int? ?? 0,
          updated: result['updated'] as int? ?? 0,
          failed: result['failed'] as int? ?? 0,
        ),
      );
    } catch (e) {
      state = state.copyWith(
        status: ImportStatus.error,
        errorMessage: e.toString(),
      );
    }
  }

  void reset() {
    state = const SpotifyImportState();
  }
}

final spotifyImportProvider =
    StateNotifierProvider<SpotifyImportNotifier, SpotifyImportState>(
  (ref) => SpotifyImportNotifier(ref.read(apiClientProvider)),
);

// ---------------------------------------------------------------------------
// API client provider
// ---------------------------------------------------------------------------

final apiClientProvider = Provider<ApiClient>((ref) => ApiClient());
