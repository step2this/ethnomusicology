import 'package:flutter/material.dart';

import '../models/refinement.dart';

class VersionHistoryPanel extends StatelessWidget {
  final List<SetlistVersion>? versions;
  final int? currentVersion;
  final bool isLoading;
  final void Function(int versionNumber) onRevert;

  const VersionHistoryPanel({
    super.key,
    required this.versions,
    required this.currentVersion,
    required this.isLoading,
    required this.onRevert,
  });

  String _formatTimestamp(String raw) {
    final dt = DateTime.tryParse(raw);
    if (dt == null) return raw;
    final local = dt.toLocal();
    final now = DateTime.now();
    final diff = now.difference(local);
    if (diff.inMinutes < 1) return 'Just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    return '${local.month}/${local.day} ${local.hour.toString().padLeft(2, '0')}:${local.minute.toString().padLeft(2, '0')}';
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Drawer(
      child: SafeArea(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                'Version History',
                style: theme.textTheme.titleMedium,
              ),
            ),
            const Divider(height: 1),
            if (isLoading)
              const Expanded(
                child: Center(child: CircularProgressIndicator()),
              )
            else if (versions == null || versions!.isEmpty)
              Expanded(
                child: Center(
                  child: Text(
                    'No version history yet',
                    style: theme.textTheme.bodyMedium?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
              )
            else
              Expanded(
                child: ListView.builder(
                  itemCount: versions!.length,
                  itemBuilder: (context, index) {
                    final version = versions![index];
                    final isCurrent = version.versionNumber == currentVersion;

                    return ListTile(
                      leading: CircleAvatar(
                        radius: 14,
                        backgroundColor: isCurrent
                            ? theme.colorScheme.primary
                            : theme.colorScheme.surfaceContainerHighest,
                        child: Text(
                          'v${version.versionNumber}',
                          style: theme.textTheme.labelSmall?.copyWith(
                            color: isCurrent
                                ? theme.colorScheme.onPrimary
                                : theme.colorScheme.onSurface,
                          ),
                        ),
                      ),
                      title: Text(
                        version.actionSummary ?? version.action ?? 'Initial version',
                        style: theme.textTheme.bodyMedium,
                      ),
                      subtitle: version.createdAt != null
                          ? Text(_formatTimestamp(version.createdAt!), style: theme.textTheme.bodySmall)
                          : null,
                      trailing: isCurrent
                          ? Chip(
                              label: const Text('Current'),
                              visualDensity: VisualDensity.compact,
                            )
                          : TextButton(
                              onPressed: () => onRevert(version.versionNumber),
                              child: const Text('Revert'),
                            ),
                      selected: isCurrent,
                    );
                  },
                ),
              ),
          ],
        ),
      ),
    );
  }
}
