import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config/constants.dart';
import '../models/setlist.dart';
import '../providers/audio_provider.dart';
import '../providers/deezer_provider.dart';
import '../providers/refinement_provider.dart';
import 'conversation_history.dart';
import 'refinement_chat_input.dart';
import 'setlist_track_tile.dart';
import 'transport_controls.dart';

class SetlistResultView extends ConsumerWidget {
  final Setlist setlist;
  final VoidCallback onArrange;

  const SetlistResultView({
    super.key,
    required this.setlist,
    required this.onArrange,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final audioState = ref.watch(audioPlaybackProvider);
    final previewState = ref.watch(previewProvider);
    final refinementState = ref.watch(refinementProvider);

    return Column(
      children: [
        // Notes
        if (setlist.notes != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              setlist.notes!,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    fontStyle: FontStyle.italic,
                  ),
            ),
          ),
        // Header row
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              Text(
                '${setlist.tracks.length} tracks',
                style: Theme.of(context).textTheme.labelLarge,
              ),
              if (refinementState.currentVersion != null) ...[
                const SizedBox(width: 8),
                Chip(
                  label: Text('v${refinementState.currentVersion}'),
                  visualDensity: VisualDensity.compact,
                ),
              ],
              if (setlist.isArranged) ...[
                const SizedBox(width: 12),
                Chip(
                  label: Text('Flow: ${setlist.harmonicFlowScoreFormatted}'),
                  avatar: const Icon(Icons.auto_awesome, size: 16),
                ),
              ],
              if (setlist.catalogPercentage != null) ...[
                const SizedBox(width: 8),
                _buildCatalogBadge(context),
              ],
              const Spacer(),
              if (refinementState.currentVersion != null && refinementState.currentVersion! > 1)
                IconButton(
                  icon: const Icon(Icons.undo),
                  tooltip: 'Undo last refinement',
                  onPressed: refinementState.isRefining
                      ? null
                      : () => ref.read(refinementProvider.notifier).refineSetlist(
                            setlist.id,
                            '!undo',
                          ),
                ),
              if (!setlist.isArranged)
                FilledButton.icon(
                  onPressed: onArrange,
                  icon: const Icon(Icons.auto_awesome),
                  label: const Text('Arrange'),
                ),
            ],
          ),
        ),
        // Transport controls
        TransportControls(
          audioState: audioState,
          trackCount: setlist.tracks.length,
          onPrevious: () => ref
              .read(audioPlaybackProvider.notifier)
              .previous(setlist.tracks, previewState),
          onTogglePause: () =>
              ref.read(audioPlaybackProvider.notifier).togglePause(),
          onStop: () => ref.read(audioPlaybackProvider.notifier).stop(),
          onNext: () => ref
              .read(audioPlaybackProvider.notifier)
              .next(setlist.tracks, previewState),
        ),
        // Catalog warning
        if (setlist.catalogWarning != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Card(
              color: Theme.of(context).colorScheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: Row(
                  children: [
                    Icon(Icons.warning_amber,
                        size: 16,
                        color: Theme.of(context).colorScheme.error),
                    const SizedBox(width: 8),
                    Expanded(child: Text(setlist.catalogWarning!)),
                  ],
                ),
              ),
            ),
          ),
        // Refinement change warning
        if (refinementState.changeWarning != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: Card(
              color: Theme.of(context).colorScheme.tertiaryContainer,
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: Row(
                  children: [
                    Icon(Icons.info_outline,
                        size: 16,
                        color: Theme.of(context).colorScheme.tertiary),
                    const SizedBox(width: 8),
                    Expanded(child: Text(refinementState.changeWarning!)),
                  ],
                ),
              ),
            ),
          ),
        // Refinement error
        if (refinementState.error != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: Card(
              color: Theme.of(context).colorScheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: Row(
                  children: [
                    Icon(Icons.error_outline,
                        size: 16,
                        color: Theme.of(context).colorScheme.error),
                    const SizedBox(width: 8),
                    Expanded(child: Text(refinementState.error!)),
                  ],
                ),
              ),
            ),
          ),
        // Track list (3/4 of remaining space)
        Expanded(
          flex: 3,
          child: ListView.builder(
            itemCount: setlist.tracks.length,
            itemBuilder: (context, index) {
              final track = setlist.tracks[index];
              final hasBpmWarning = setlist.bpmWarnings.any((w) =>
                  w.fromPosition == track.position ||
                  w.toPosition == track.position);
              final trackKey = previewKey(track);
              final trackInfo = previewState.trackInfo[trackKey];

              final isCurrentTrack = audioState.currentTrackIndex == index;
              return SetlistTrackTile(
                track: track,
                hasBpmWarning: hasBpmWarning,
                isPlaying: isCurrentTrack &&
                    (audioState.isPlaying || audioState.isPaused),
                isPaused: audioState.isPaused && isCurrentTrack,
                isLoading: audioState.isLoading && isCurrentTrack,
                hasPreview: previewState.hasPreview(trackKey),
                previewStatus: trackInfo?.status,
                previewSearchQueries: trackInfo?.searchQueries,
                previewSource: trackInfo?.source,
                externalUrl: trackInfo?.externalUrl,
                spotifyUri: track.spotifyUri,
                onPlay: () => ref
                    .read(audioPlaybackProvider.notifier)
                    .playFromIndex(index, setlist.tracks, previewState),
                onPause: () =>
                    ref.read(audioPlaybackProvider.notifier).togglePause(),
              );
            },
          ),
        ),
        const Divider(height: 1),
        // Conversation history (1/4 of remaining space)
        Expanded(
          flex: 1,
          child: ConversationHistory(
            messages: refinementState.conversation,
            isRefining: refinementState.isRefining,
          ),
        ),
        // Chat input
        RefinementChatInput(
          setlistId: setlist.id,
          isRefining: refinementState.isRefining,
        ),
      ],
    );
  }

  Widget _buildCatalogBadge(BuildContext context) {
    final pct = setlist.catalogPercentage!;
    final isLow = pct < AppConstants.lowCatalogThreshold;
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: isLow
            ? theme.colorScheme.errorContainer
            : theme.colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        'Catalog: ${pct.round()}%',
        style: theme.textTheme.labelSmall?.copyWith(
          color: isLow
              ? theme.colorScheme.onErrorContainer
              : theme.colorScheme.onPrimaryContainer,
        ),
      ),
    );
  }
}
