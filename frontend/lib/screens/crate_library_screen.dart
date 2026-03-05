import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../config/routes.dart';
import '../providers/crate_provider.dart';

class CrateLibraryScreen extends ConsumerStatefulWidget {
  const CrateLibraryScreen({super.key});

  @override
  ConsumerState<CrateLibraryScreen> createState() => _CrateLibraryScreenState();
}

class _CrateLibraryScreenState extends ConsumerState<CrateLibraryScreen> {
  @override
  void initState() {
    super.initState();
    Future.microtask(
        () => ref.read(crateLibraryProvider.notifier).loadCrates());
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(crateLibraryProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('My Crates'),
        centerTitle: true,
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _showCreateDialog(context),
        icon: const Icon(Icons.add),
        label: const Text('New Crate'),
      ),
      body: _buildBody(context, state),
    );
  }

  Widget _buildBody(BuildContext context, CrateLibraryState state) {
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
                  ref.read(crateLibraryProvider.notifier).loadCrates(),
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }
    if (state.crates.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.inbox, size: 64),
            SizedBox(height: 16),
            Text('No crates yet.\nCreate one to organize your tracks.',
                textAlign: TextAlign.center),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 88),
      itemCount: state.crates.length,
      itemBuilder: (context, index) {
        final c = state.crates[index];
        return Card(
          margin: const EdgeInsets.only(bottom: 8),
          child: ListTile(
            title:
                Text(c.name, maxLines: 1, overflow: TextOverflow.ellipsis),
            subtitle: Text('${c.trackCount} tracks'),
            leading: const Icon(Icons.inbox),
            trailing: IconButton(
              icon: const Icon(Icons.delete_outline),
              onPressed: () => _showDeleteDialog(context, c.id, c.name),
              tooltip: 'Delete crate',
            ),
            onTap: () => context.go('${AppRoutes.crateDetail}/${c.id}'),
          ),
        );
      },
    );
  }

  void _showCreateDialog(BuildContext context) {
    final controller = TextEditingController();
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('New Crate'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(labelText: 'Crate name'),
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
                ref.read(crateLibraryProvider.notifier).createCrate(name);
              }
              Navigator.of(ctx).pop();
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }

  void _showDeleteDialog(BuildContext context, String id, String name) {
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete Crate'),
        content: Text('Delete "$name"? This cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              ref.read(crateLibraryProvider.notifier).deleteCrate(id);
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
}
