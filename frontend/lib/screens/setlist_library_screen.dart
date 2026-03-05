import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../config/routes.dart';
import '../providers/setlist_library_provider.dart';

class SetlistLibraryScreen extends ConsumerStatefulWidget {
  const SetlistLibraryScreen({super.key});

  @override
  ConsumerState<SetlistLibraryScreen> createState() =>
      _SetlistLibraryScreenState();
}

class _SetlistLibraryScreenState extends ConsumerState<SetlistLibraryScreen> {
  @override
  void initState() {
    super.initState();
    Future.microtask(
        () => ref.read(setlistLibraryProvider.notifier).loadSetlists());
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(setlistLibraryProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('My Setlists'),
        centerTitle: true,
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.go(AppRoutes.setlistGenerate),
        icon: const Icon(Icons.add),
        label: const Text('New Setlist'),
      ),
      body: _buildBody(context, state),
    );
  }

  Widget _buildBody(BuildContext context, SetlistLibraryState state) {
    if (state.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (state.error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(state.error!),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: () =>
                  ref.read(setlistLibraryProvider.notifier).loadSetlists(),
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }
    if (state.setlists.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.queue_music, size: 64),
            SizedBox(height: 16),
            Text('No saved setlists yet.\nGenerate one to get started.',
                textAlign: TextAlign.center),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 88),
      itemCount: state.setlists.length,
      itemBuilder: (context, index) {
        final s = state.setlists[index];
        return Card(
          margin: const EdgeInsets.only(bottom: 8),
          child: ListTile(
            title: Text(s.name ?? s.prompt,
                maxLines: 1, overflow: TextOverflow.ellipsis),
            subtitle: Text('${s.trackCount} tracks · ${_formatDate(s.createdAt)}'),
            leading: const Icon(Icons.queue_music),
            trailing: PopupMenuButton<_SetlistAction>(
              onSelected: (action) => _handleAction(context, action, s),
              itemBuilder: (_) => [
                const PopupMenuItem(
                  value: _SetlistAction.rename,
                  child: Text('Rename'),
                ),
                const PopupMenuItem(
                  value: _SetlistAction.duplicate,
                  child: Text('Duplicate'),
                ),
                const PopupMenuItem(
                  value: _SetlistAction.delete,
                  child: Text('Delete'),
                ),
              ],
            ),
            onTap: () =>
                context.go('${AppRoutes.setlistView}/${s.id}'),
          ),
        );
      },
    );
  }

  void _handleAction(
      BuildContext context, _SetlistAction action, SetlistSummary s) {
    switch (action) {
      case _SetlistAction.rename:
        _showRenameDialog(context, s);
      case _SetlistAction.duplicate:
        ref.read(setlistLibraryProvider.notifier).duplicateSetlist(s.id);
      case _SetlistAction.delete:
        _showDeleteDialog(context, s);
    }
  }

  void _showRenameDialog(BuildContext context, SetlistSummary s) {
    final controller = TextEditingController(text: s.name ?? s.prompt);
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Rename Setlist'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(labelText: 'Name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              final name = controller.text.trim();
              if (name.isNotEmpty) {
                ref
                    .read(setlistLibraryProvider.notifier)
                    .renameSetlist(s.id, name);
              }
              Navigator.of(ctx).pop();
            },
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }

  void _showDeleteDialog(BuildContext context, SetlistSummary s) {
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete Setlist'),
        content:
            Text('Delete "${s.name ?? s.prompt}"? This cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              ref.read(setlistLibraryProvider.notifier).deleteSetlist(s.id);
              Navigator.of(ctx).pop();
            },
            style: FilledButton.styleFrom(
              backgroundColor: Theme.of(context).colorScheme.error,
            ),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  String _formatDate(String iso) {
    try {
      final dt = DateTime.parse(iso);
      return '${dt.month}/${dt.day}/${dt.year}';
    } catch (_) {
      return iso;
    }
  }
}

enum _SetlistAction { rename, duplicate, delete }
