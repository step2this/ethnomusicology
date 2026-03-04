import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../models/setlist_track.dart';
import '../providers/deezer_provider.dart';

class SetlistTrackTile extends StatelessWidget {
  final SetlistTrack track;
  final bool hasBpmWarning;
  final VoidCallback? onPlay;
  final VoidCallback? onPause;
  final bool isPlaying;
  final bool isPaused;
  final bool isLoading;
  final bool hasPreview;
  final PreviewSearchStatus? previewStatus;
  final List<String>? previewSearchQueries;
  final String? previewSource;
  final String? externalUrl;
  final String? spotifyUri;

  const SetlistTrackTile({
    super.key,
    required this.track,
    this.hasBpmWarning = false,
    this.onPlay,
    this.onPause,
    this.isPlaying = false,
    this.isPaused = false,
    this.isLoading = false,
    this.hasPreview = false,
    this.previewStatus,
    this.previewSearchQueries,
    this.previewSource,
    this.externalUrl,
    this.spotifyUri,
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
                        child: InkWell(
                          onTap: () => _searchGoogle(track.title, track.artist),
                          child: Text(
                            track.title,
                            style: theme.textTheme.bodyLarge?.copyWith(
                              fontWeight: FontWeight.w600,
                              decoration: TextDecoration.underline,
                            ),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ),
                      if (spotifyUri != null)
                        Tooltip(
                          message: 'Open in Spotify',
                          child: InkWell(
                            onTap: () => _openSpotify(spotifyUri!),
                            borderRadius: BorderRadius.circular(4),
                            child: Padding(
                              padding: const EdgeInsets.symmetric(horizontal: 4),
                              child: Icon(
                                Icons.open_in_new,
                                size: 14,
                                color: theme.colorScheme.primary,
                              ),
                            ),
                          ),
                        ),
                      if (externalUrl != null && previewSource == 'itunes')
                        Tooltip(
                          message: 'Open in Apple Music',
                          child: InkWell(
                            onTap: () => launchUrl(
                              Uri.parse(externalUrl!),
                              mode: LaunchMode.externalApplication,
                            ),
                            borderRadius: BorderRadius.circular(4),
                            child: Padding(
                              padding: const EdgeInsets.symmetric(horizontal: 4),
                              child: Icon(
                                Icons.apple,
                                size: 14,
                                color: Colors.grey.shade700,
                              ),
                            ),
                          ),
                        ),
                      _sourceBadge(context),
                    ],
                  ),
                  const SizedBox(height: 2),
                  InkWell(
                    onTap: () => _searchGoogleArtist(track.artist),
                    child: Text(
                      track.artist,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                        decoration: TextDecoration.underline,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
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
            const SizedBox(width: 8),

            // Deezer search status dot
            _previewStatusDot(),
            const SizedBox(width: 4),

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
                        (isPlaying && !isPaused)
                            ? Icons.pause
                            : Icons.play_arrow,
                        color: theme.colorScheme.primary,
                      ),
                onPressed: isLoading
                    ? null
                    : (isPlaying ? onPause : onPlay),
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

  void _searchGoogle(String title, String artist) {
    final query = Uri.encodeComponent('"$title" "$artist"');
    launchUrl(
      Uri.parse('https://www.google.com/search?q=$query'),
      mode: LaunchMode.externalApplication,
    );
  }

  void _searchGoogleArtist(String artist) {
    final query = Uri.encodeComponent('"$artist" music');
    launchUrl(
      Uri.parse('https://www.google.com/search?q=$query'),
      mode: LaunchMode.externalApplication,
    );
  }

  void _openSpotify(String uri) {
    // uri format: spotify:track:XXXX — extract the last segment
    final id = uri.split(':').last;
    launchUrl(
      Uri.parse('https://open.spotify.com/track/$id'),
      mode: LaunchMode.externalApplication,
    );
  }

  Widget _previewStatusDot() {
    final status = previewStatus;
    if (status == null) return const SizedBox.shrink();

    final queries = previewSearchQueries?.join(' → ') ?? '';

    IconData icon;
    Color color;
    String tooltipLabel;
    switch (status) {
      case PreviewSearchStatus.loading:
        return Tooltip(
          message: 'Searching: $queries',
          child: const SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        );
      case PreviewSearchStatus.found:
        final src = previewSource ?? 'unknown';
        if (src == 'itunes') {
          icon = Icons.apple;
          color = Colors.grey.shade700;
          tooltipLabel = 'iTunes';
        } else {
          icon = Icons.check_circle;
          color = Colors.green;
          tooltipLabel = 'Deezer';
        }
        return Tooltip(
          message: '$tooltipLabel: $queries',
          child: Icon(icon, size: 16, color: color),
        );
      case PreviewSearchStatus.notFound:
        icon = Icons.cancel;
        color = Colors.red;
      case PreviewSearchStatus.error:
        icon = Icons.error;
        color = Colors.orange;
    }

    return Tooltip(
      message: 'No preview: $queries',
      child: Icon(icon, size: 16, color: color),
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
