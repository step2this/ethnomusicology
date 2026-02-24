import 'package:flutter/material.dart';
import '../providers/spotify_import_provider.dart';

class ImportSummaryCard extends StatelessWidget {
  final ImportProgress progress;

  const ImportSummaryCard({super.key, required this.progress});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      color: theme.colorScheme.primaryContainer,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.check_circle,
                    color: theme.colorScheme.primary, size: 28),
                const SizedBox(width: 12),
                Text(
                  'Import Complete',
                  style: theme.textTheme.titleMedium?.copyWith(
                    color: theme.colorScheme.onPrimaryContainer,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text(
              'Imported ${progress.total} tracks',
              style: theme.textTheme.headlineSmall?.copyWith(
                color: theme.colorScheme.onPrimaryContainer,
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                _StatChip(
                  label: 'New',
                  count: progress.inserted,
                  color: Colors.green,
                ),
                const SizedBox(width: 8),
                _StatChip(
                  label: 'Updated',
                  count: progress.updated,
                  color: Colors.blue,
                ),
                const SizedBox(width: 8),
                if (progress.failed > 0)
                  _StatChip(
                    label: 'Failed',
                    count: progress.failed,
                    color: Colors.red,
                  ),
              ],
            ),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: () {
                // Navigate to catalog - will be wired in UC-003
              },
              icon: const Icon(Icons.library_music),
              label: const Text('View Catalog'),
            ),
          ],
        ),
      ),
    );
  }
}

class _StatChip extends StatelessWidget {
  final String label;
  final int count;
  final Color color;

  const _StatChip({
    required this.label,
    required this.count,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Chip(
      avatar: CircleAvatar(
        backgroundColor: color,
        radius: 10,
        child: Text(
          '$count',
          style: const TextStyle(color: Colors.white, fontSize: 10),
        ),
      ),
      label: Text(label),
    );
  }
}
