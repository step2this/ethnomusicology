import 'package:flutter/material.dart';

import '../config/theme.dart';
import '../models/track.dart';

class TrackTile extends StatelessWidget {
  final Track track;
  final VoidCallback? onTap;

  const TrackTile({
    super.key,
    required this.track,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: _buildAlbumArt(context),
      title: Text(
        track.title,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(
        track.artist,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildMetadataChip(context, track.bpmFormatted, 'BPM'),
          const SizedBox(width: 8),
          _buildMetadataChip(context, track.keyFormatted, 'KEY'),
          const SizedBox(width: 8),
          Text(
            track.durationFormatted,
            style: context.textTheme.bodySmall?.copyWith(
              color: context.colors.outline,
            ),
          ),
        ],
      ),
      onTap: onTap,
    );
  }

  Widget _buildAlbumArt(BuildContext context) {
    if (track.albumArtUrl != null) {
      return ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: Image.network(
          track.albumArtUrl!,
          width: 48,
          height: 48,
          fit: BoxFit.cover,
          errorBuilder: (_, _, _) => _buildPlaceholder(context),
        ),
      );
    }
    return _buildPlaceholder(context);
  }

  Widget _buildPlaceholder(BuildContext context) {
    return Container(
      width: 48,
      height: 48,
      decoration: BoxDecoration(
        color: context.colors.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Icon(
        Icons.music_note,
        color: context.colors.outline,
      ),
    );
  }

  Widget _buildMetadataChip(BuildContext context, String value, String label) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          value,
          style: context.textTheme.bodySmall?.copyWith(
            fontWeight: FontWeight.bold,
          ),
        ),
        Text(
          label,
          style: context.textTheme.labelSmall?.copyWith(
            color: context.colors.outline,
            fontSize: 9,
          ),
        ),
      ],
    );
  }
}
