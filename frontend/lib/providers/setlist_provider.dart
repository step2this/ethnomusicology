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

  const SetlistState({
    this.setlist,
    this.isGenerating = false,
    this.isArranging = false,
    this.error,
  });

  SetlistState copyWith({
    Setlist? Function()? setlist,
    bool? isGenerating,
    bool? isArranging,
    String? Function()? error,
  }) {
    return SetlistState(
      setlist: setlist != null ? setlist() : this.setlist,
      isGenerating: isGenerating ?? this.isGenerating,
      isArranging: isArranging ?? this.isArranging,
      error: error != null ? error() : this.error,
    );
  }

  bool get hasSetlist => setlist != null;
  bool get isLoading => isGenerating || isArranging;
}

class SetlistNotifier extends StateNotifier<SetlistState> {
  final ApiClient _apiClient;

  SetlistNotifier(this._apiClient) : super(const SetlistState());

  Future<void> generateSetlist(String prompt, {int? trackCount}) async {
    state = const SetlistState(isGenerating: true);
    try {
      final setlist = await _apiClient.generateSetlist(
        prompt,
        trackCount: trackCount,
      );
      state = SetlistState(setlist: setlist);
    } catch (e) {
      state = SetlistState(
        error: _parseError(e),
      );
    }
  }

  Future<void> arrangeSetlist() async {
    if (state.setlist == null) return;

    state = state.copyWith(isArranging: true, error: () => null);
    try {
      final arranged = await _apiClient.arrangeSetlist(state.setlist!.id);
      state = SetlistState(setlist: arranged);
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
    if (e.toString().contains('EMPTY_CATALOG')) {
      return 'No tracks in your catalog. Import music first.';
    }
    if (e.toString().contains('LLM_ERROR')) {
      return 'AI service temporarily unavailable. Please try again.';
    }
    return 'Something went wrong. Please try again.';
  }
}

final setlistProvider =
    StateNotifierProvider<SetlistNotifier, SetlistState>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return SetlistNotifier(apiClient);
});
