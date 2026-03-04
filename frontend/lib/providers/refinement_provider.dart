import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/refinement.dart';
import 'api_provider.dart';
import 'audio_provider.dart';
import 'deezer_provider.dart';
import 'setlist_provider.dart';

class RefinementState {
  final List<ConversationMessage> conversation;
  final int? currentVersion;
  final bool isRefining;
  final String? error;
  final String? changeWarning;
  final List<SetlistVersion>? versionHistory;
  final bool isLoadingHistory;

  const RefinementState({
    this.conversation = const [],
    this.currentVersion,
    this.isRefining = false,
    this.error,
    this.changeWarning,
    this.versionHistory,
    this.isLoadingHistory = false,
  });

  RefinementState copyWith({
    List<ConversationMessage>? conversation,
    int? Function()? currentVersion,
    bool? isRefining,
    String? Function()? error,
    String? Function()? changeWarning,
    List<SetlistVersion>? Function()? versionHistory,
    bool? isLoadingHistory,
  }) {
    return RefinementState(
      conversation: conversation ?? this.conversation,
      currentVersion: currentVersion != null ? currentVersion() : this.currentVersion,
      isRefining: isRefining ?? this.isRefining,
      error: error != null ? error() : this.error,
      changeWarning: changeWarning != null ? changeWarning() : this.changeWarning,
      versionHistory: versionHistory != null ? versionHistory() : this.versionHistory,
      isLoadingHistory: isLoadingHistory ?? this.isLoadingHistory,
    );
  }
}

class RefinementNotifier extends Notifier<RefinementState> {
  static int _messageCounter = 0;

  @override
  RefinementState build() => const RefinementState();

  Future<void> refineSetlist(String setlistId, String message) async {
    // H1: Stop audio BEFORE making the API call
    ref.read(audioPlaybackProvider.notifier).stop();

    // Add optimistic user message
    final optimisticMsg = ConversationMessage(
      id: 'pending-${++_messageCounter}',
      setlistId: setlistId,
      role: 'user',
      content: message,
    );

    state = state.copyWith(
      isRefining: true,
      error: () => null,
      conversation: [...state.conversation, optimisticMsg],
    );

    try {
      final response = await ref.read(apiClientProvider).refineSetlist(setlistId, message);

      // Add assistant message
      final assistantMsg = ConversationMessage(
        id: 'assistant-${++_messageCounter}',
        setlistId: setlistId,
        role: 'assistant',
        content: response.explanation,
      );

      // Optimistically add version to local history
      final newVersion = SetlistVersion(
        id: 'local-${++_messageCounter}',
        setlistId: setlistId,
        versionNumber: response.versionNumber,
        action: 'refine',
        actionSummary: message.length > 50
            ? '${message.substring(0, 50)}...'
            : message,
      );

      state = state.copyWith(
        isRefining: false,
        currentVersion: () => response.versionNumber,
        conversation: [...state.conversation, assistantMsg],
        changeWarning: () => response.changeWarning,
        versionHistory: () => [...(state.versionHistory ?? []), newVersion],
      );

      // Push tracks to setlist provider
      ref.read(setlistProvider.notifier).updateTracks(response.tracks);

      // Only prefetch tracks not already cached
      final deezerState = ref.read(deezerPreviewProvider);
      final uncachedTracks = response.tracks.where((t) {
        final key = t.trackId ?? 'unknown-${t.position}';
        return !deezerState.trackInfo.containsKey(key);
      }).toList();
      if (uncachedTracks.isNotEmpty) {
        ref.read(deezerPreviewProvider.notifier).prefetchForSetlist(uncachedTracks);
      }
    } on Exception catch (e) {
      // Remove optimistic message on error
      final restored = state.conversation.where((m) => m.id != optimisticMsg.id).toList();
      state = state.copyWith(
        isRefining: false,
        error: () => _parseError(e),
        conversation: restored,
      );
    }
  }

  Future<void> revertToVersion(String setlistId, int versionNumber) async {
    ref.read(audioPlaybackProvider.notifier).stop();

    state = state.copyWith(isRefining: true, error: () => null);

    try {
      final response = await ref.read(apiClientProvider).revertSetlist(setlistId, versionNumber);

      final systemMsg = ConversationMessage(
        id: 'assistant-revert-${++_messageCounter}',
        setlistId: setlistId,
        role: 'assistant',
        content: 'Reverted to version $versionNumber. ${response.explanation}',
      );

      // Optimistically add revert version to local history
      final revertVersion = SetlistVersion(
        id: 'local-revert-${++_messageCounter}',
        setlistId: setlistId,
        versionNumber: response.versionNumber,
        action: 'revert',
        actionSummary: 'Reverted to v$versionNumber',
      );

      state = state.copyWith(
        isRefining: false,
        currentVersion: () => response.versionNumber,
        conversation: [...state.conversation, systemMsg],
        changeWarning: () => response.changeWarning,
        versionHistory: () => [...(state.versionHistory ?? []), revertVersion],
      );

      ref.read(setlistProvider.notifier).updateTracks(response.tracks);

      // Only prefetch tracks not already cached
      final deezerState = ref.read(deezerPreviewProvider);
      final uncachedTracks = response.tracks.where((t) {
        final key = t.trackId ?? 'unknown-${t.position}';
        return !deezerState.trackInfo.containsKey(key);
      }).toList();
      if (uncachedTracks.isNotEmpty) {
        ref.read(deezerPreviewProvider.notifier).prefetchForSetlist(uncachedTracks);
      }
    } on Exception catch (e) {
      state = state.copyWith(
        isRefining: false,
        error: () => _parseError(e),
      );
    }
  }

  Future<void> loadHistory(String setlistId) async {
    state = state.copyWith(isLoadingHistory: true);

    try {
      final history = await ref.read(apiClientProvider).getSetlistHistory(setlistId);

      state = state.copyWith(
        isLoadingHistory: false,
        versionHistory: () => history.versions,
        conversation: history.conversations,
        currentVersion: () => history.versions.isNotEmpty
            ? history.versions.map((v) => v.versionNumber).reduce((a, b) => a > b ? a : b)
            : null,
      );
    } on Exception catch (_) {
      state = state.copyWith(isLoadingHistory: false);
    }
  }

  void reset() {
    state = const RefinementState();
  }

  String _parseError(dynamic e) {
    final msg = e.toString();
    if (msg.contains('TURN_LIMIT_EXCEEDED')) {
      return 'Refinement limit reached. Start a new setlist to continue.';
    }
    if (msg.contains('NOT_FOUND')) {
      return 'Setlist not found. It may have been deleted.';
    }
    if (msg.contains('LLM_ERROR')) {
      return 'AI service temporarily unavailable. Please try again.';
    }
    if (msg.contains('INVALID_REQUEST')) {
      return 'Invalid request. Please try a different message.';
    }
    return 'Refinement failed. Please try again.';
  }
}

final refinementProvider =
    NotifierProvider<RefinementNotifier, RefinementState>(RefinementNotifier.new);
