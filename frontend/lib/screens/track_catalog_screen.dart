import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../config/theme.dart';
import '../providers/track_catalog_provider.dart';
import '../widgets/track_tile.dart';

class TrackCatalogScreen extends ConsumerStatefulWidget {
  const TrackCatalogScreen({super.key});

  @override
  ConsumerState<TrackCatalogScreen> createState() => _TrackCatalogScreenState();
}

class _TrackCatalogScreenState extends ConsumerState<TrackCatalogScreen> {
  final _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    // Load first page on mount
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(trackCatalogProvider.notifier).loadFirstPage();
    });
    _scrollController.addListener(_onScroll);
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (_scrollController.position.pixels >=
        _scrollController.position.maxScrollExtent - 200) {
      ref.read(trackCatalogProvider.notifier).loadNextPage();
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(trackCatalogProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Track Catalog'),
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.sort),
            tooltip: 'Sort by',
            onSelected: (value) {
              final parts = value.split(':');
              ref
                  .read(trackCatalogProvider.notifier)
                  .setSort(parts[0], parts[1]);
            },
            itemBuilder: (context) => [
              const PopupMenuItem(
                  value: 'date_added:desc', child: Text('Newest first')),
              const PopupMenuItem(
                  value: 'date_added:asc', child: Text('Oldest first')),
              const PopupMenuItem(
                  value: 'title:asc', child: Text('Title A-Z')),
              const PopupMenuItem(
                  value: 'title:desc', child: Text('Title Z-A')),
              const PopupMenuItem(
                  value: 'bpm:asc', child: Text('BPM (low to high)')),
              const PopupMenuItem(
                  value: 'bpm:desc', child: Text('BPM (high to low)')),
            ],
          ),
        ],
      ),
      body: _buildBody(context, state),
    );
  }

  Widget _buildBody(BuildContext context, TrackCatalogState state) {
    // Error state
    if (state.error != null && state.tracks.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.error_outline, size: 48, color: context.colors.error),
            const SizedBox(height: 16),
            Text(state.error!, style: context.textTheme.bodyLarge),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: () => ref.read(trackCatalogProvider.notifier).retry(),
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    // Loading state (initial)
    if (state.isLoading && state.tracks.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }

    // Empty state
    if (!state.isLoading && state.tracks.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.library_music_outlined,
                size: 64, color: context.colors.outline),
            const SizedBox(height: 16),
            Text(
              'No tracks yet',
              style: context.textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            Text(
              'Import from Spotify to get started',
              style: context.textTheme.bodyMedium?.copyWith(
                color: context.colors.outline,
              ),
            ),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: () => context.go('/import/spotify'),
              icon: const Icon(Icons.download),
              label: const Text('Import from Spotify'),
            ),
          ],
        ),
      );
    }

    // Track list
    return ListView.builder(
      controller: _scrollController,
      itemCount: state.tracks.length + (state.hasMore ? 1 : 0),
      itemBuilder: (context, index) {
        if (index >= state.tracks.length) {
          return const Padding(
            padding: EdgeInsets.all(16),
            child: Center(child: CircularProgressIndicator()),
          );
        }
        return TrackTile(track: state.tracks[index]);
      },
    );
  }
}
