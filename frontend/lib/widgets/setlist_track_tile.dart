import 'package:flutter/material.dart';

import '../models/setlist_track.dart';

class SetlistTrackTile extends StatelessWidget {
  final SetlistTrack track;
  final bool hasBpmWarning;
  final VoidCallback? onPlay;
  final VoidCallback? onStop;
  final bool isPlaying;
  final bool isLoading;
  final bool hasPreview;

  const SetlistTrackTile({
    super.key,
    required this.track,
    this.hasBpmWarning = false,
    this.onPlay,
    this.onStop,
    this.isPlaying = false,
    this.isLoading = false,
    this.hasPreview = false,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            // Position number
            SizedBox(
              width: 32,
              child: Text(
                '${track.position}',
                style: theme.textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                  color: theme.colorScheme.primary,
                ),
                textAlign: TextAlign.center,
              ),
            ),
            const SizedBox(width: 12),

            // Track info
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          track.title,
                          style: theme.textTheme.bodyLarge?.copyWith(
                            fontWeight: FontWeight.w600,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      _sourceBadge(context),
                    ],
                  ),
                  const SizedBox(height: 2),
                  Text(
                    track.artist,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 6),
                  // Metadata row
                  Row(
                    children: [
                      _metadataChip(context, 'BPM', track.bpmFormatted),
                      const SizedBox(width: 8),
                      _metadataChip(context, 'Key', track.camelotFormatted),
                      const SizedBox(width: 8),
                      _metadataChip(context, 'Energy', track.energyFormatted),
                      if (track.hasTransitionScore) ...[
                        const SizedBox(width: 8),
                        _metadataChip(
                          context,
                          'Flow',
                          track.transitionScoreFormatted,
                        ),
                      ],
                    ],
                  ),
                  if (hasBpmWarning) ...[
                    const SizedBox(height: 4),
                    Row(
                      children: [
                        Icon(Icons.warning_amber,
                            size: 14, color: theme.colorScheme.error),
                        const SizedBox(width: 4),
                        Text(
                          'Large BPM jump',
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: theme.colorScheme.error,
                          ),
                        ),
                      ],
                    ),
                  ],
                  if (track.transitionNote != null) ...[
                    const SizedBox(height: 4),
                    Text(
                      track.transitionNote!,
                      style: theme.textTheme.bodySmall?.copyWith(
                        fontStyle: FontStyle.italic,
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ],
                ],
              ),
            ),
            const SizedBox(width: 12),

            // Play/Stop button
            if (hasPreview)
              IconButton(
                icon: isLoading
                    ? SizedBox(
                        width: 24,
                        height: 24,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          valueColor: AlwaysStoppedAnimation<Color>(
                            theme.colorScheme.primary,
                          ),
                        ),
                      )
                    : Icon(
                        isPlaying ? Icons.stop : Icons.play_arrow,
                        color: theme.colorScheme.primary,
                      ),
                onPressed: isPlaying ? onStop : onPlay,
              )
            else
              Tooltip(
                message: 'No preview available',
                child: IconButton(
                  icon: Icon(
                    Icons.music_off,
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                  onPressed: null,
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _sourceBadge(BuildContext context) {
    final theme = Theme.of(context);
    final isCatalog = track.isCatalogTrack;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: isCatalog
            ? theme.colorScheme.primaryContainer
            : theme.colorScheme.tertiaryContainer,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        isCatalog ? 'Catalog' : 'Suggestion',
        style: theme.textTheme.labelSmall?.copyWith(
          color: isCatalog
              ? theme.colorScheme.onPrimaryContainer
              : theme.colorScheme.onTertiaryContainer,
        ),
      ),
    );
  }

  Widget _metadataChip(BuildContext context, String label, String value) {
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        '$label: $value',
        style: theme.textTheme.labelSmall,
      ),
    );
  }
}
