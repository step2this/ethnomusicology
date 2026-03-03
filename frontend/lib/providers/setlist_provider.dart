import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist.dart';
import '../services/api_client.dart';
import 'api_provider.dart';

// State for setlist generation flow
class SetlistState {
  final Setlist? setlist;
  final bool isGenerating;
  final bool isArranging;
  final String? error;
  final String? energyProfile;
  final bool creativeMode;
  final String? sourcePlaylistId;

  const SetlistState({
    this.setlist,
    this.isGenerating = false,
    this.isArranging = false,
    this.error,
    this.energyProfile,
    this.creativeMode = false,
    this.sourcePlaylistId,
  });

  SetlistState copyWith({
    Setlist? Function()? setlist,
    bool? isGenerating,
    bool? isArranging,
    String? Function()? error,
    String? Function()? energyProfile,
    bool? creativeMode,
    String? Function()? sourcePlaylistId,
  }) {
    return SetlistState(
      setlist: setlist != null ? setlist() : this.setlist,
      isGenerating: isGenerating ?? this.isGenerating,
      isArranging: isArranging ?? this.isArranging,
      error: error != null ? error() : this.error,
      energyProfile:
          energyProfile != null ? energyProfile() : this.energyProfile,
      creativeMode: creativeMode ?? this.creativeMode,
      sourcePlaylistId: sourcePlaylistId != null
          ? sourcePlaylistId()
          : this.sourcePlaylistId,
    );
  }

  bool get hasSetlist => setlist != null;
  bool get isLoading => isGenerating || isArranging;
}

class SetlistNotifier extends StateNotifier<SetlistState> {
  final ApiClient _apiClient;

  SetlistNotifier(this._apiClient) : super(const SetlistState());

  Future<void> generateSetlist(
    String prompt, {
    int? trackCount,
    String? energyProfile,
    String? sourcePlaylistId,
    String? seedTracklist,
    bool? creativeMode,
    double? bpmMin,
    double? bpmMax,
  }) async {
    state = SetlistState(
      isGenerating: true,
      energyProfile: energyProfile,
      creativeMode: creativeMode ?? false,
      sourcePlaylistId: sourcePlaylistId,
    );
    try {
      final setlist = await _apiClient.generateSetlist(
        prompt,
        trackCount: trackCount,
        energyProfile: energyProfile,
        sourcePlaylistId: sourcePlaylistId,
        seedTracklist: seedTracklist,
        creativeMode: creativeMode,
        bpmMin: bpmMin,
        bpmMax: bpmMax,
      );
      state = state.copyWith(
        setlist: () => setlist,
        isGenerating: false,
      );
    } catch (e) {
      state = state.copyWith(
        isGenerating: false,
        error: () => _parseError(e),
      );
    }
  }

  Future<void> arrangeSetlist() async {
    if (state.setlist == null) return;

    state = state.copyWith(isArranging: true, error: () => null);
    try {
      final arranged = await _apiClient.arrangeSetlist(
        state.setlist!.id,
        energyProfile: state.energyProfile,
      );
      state = state.copyWith(
        setlist: () => arranged,
        isArranging: false,
      );
    } catch (e) {
      state = state.copyWith(
        isArranging: false,
        error: () => _parseError(e),
      );
    }
  }

  void reset() {
    state = const SetlistState();
  }

  String _parseError(dynamic e) {
    final msg = e.toString();
    if (msg.contains('EMPTY_CATALOG')) {
      return 'No tracks in your catalog. Import music first.';
    }
    if (msg.contains('LLM_ERROR')) {
      return 'AI service temporarily unavailable. Please try again.';
    }
    if (msg.contains('INVALID_ENERGY_PROFILE')) {
      return 'Invalid energy profile selected.';
    }
    if (msg.contains('PLAYLIST_NOT_FOUND')) {
      return 'Playlist not found. Please check the URL and try again.';
    }
    if (msg.contains('INVALID_BPM_RANGE')) {
      return 'Invalid BPM range. Min must be 60-200 and less than max.';
    }
    return 'Something went wrong. Please try again.';
  }
}

final setlistProvider =
    StateNotifierProvider<SetlistNotifier, SetlistState>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return SetlistNotifier(apiClient);
});
