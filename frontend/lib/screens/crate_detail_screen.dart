import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/crate.dart';
import '../providers/api_provider.dart';
import '../providers/crate_provider.dart';
import '../providers/setlist_library_provider.dart';

class CrateDetailScreen extends ConsumerStatefulWidget {
  final String crateId;

  const CrateDetailScreen({super.key, required this.crateId});

  @override
  ConsumerState<CrateDetailScreen> createState() => _CrateDetailScreenState();
}

class _CrateDetailScreenState extends ConsumerState<CrateDetailScreen> {
  CrateDetail? _detail;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final detail =
          await ref.read(apiClientProvider).getCrate(widget.crateId);
      if (mounted) {
        setState(() {
          _detail = detail;
          _isLoading = false;
        });
      }
    } on Exception catch (e) {
      if (mounted) {
        setState(() {
          _error = 'Failed to load crate: $e';
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(_detail?.name ?? 'Crate'),
        centerTitle: true,
        actions: [
          if (_detail != null)
            IconButton(
              icon: const Icon(Icons.playlist_add),
              onPressed: () => _showAddFromSetlistDialog(context),
              tooltip: 'Add from setlist',
            ),
        ],
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(_error!),
            const SizedBox(height: 16),
            OutlinedButton(onPressed: _load, child: const Text('Retry')),
          ],
        ),
      );
    }
    final detail = _detail!;
    if (detail.tracks.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.music_note, size: 64),
            SizedBox(height: 16),
            Text('No tracks yet.\nAdd tracks from a setlist.',
                textAlign: TextAlign.center),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      itemCount: detail.tracks.length,
      itemBuilder: (context, index) {
        final track = detail.tracks[index];
        return Card(
          margin: const EdgeInsets.only(bottom: 6),
          child: ListTile(
            leading: CircleAvatar(child: Text('${track.position}')),
            title: Text(track.title,
                maxLines: 1, overflow: TextOverflow.ellipsis),
            subtitle: Text(track.artist),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (track.bpm != null)
                  Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: Text(
                      '${track.bpm!.round()} BPM',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                IconButton(
                  icon: const Icon(Icons.remove_circle_outline),
                  onPressed: () => _removeTrack(track.id),
                  tooltip: 'Remove from crate',
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Future<void> _removeTrack(String trackId) async {
    try {
      await ref
          .read(apiClientProvider)
          .removeCrateTrack(widget.crateId, trackId);
      setState(() {
        _detail = CrateDetail(
          id: _detail!.id,
          name: _detail!.name,
          tracks:
              _detail!.tracks.where((t) => t.id != trackId).toList(),
          createdAt: _detail!.createdAt,
          updatedAt: _detail!.updatedAt,
        );
      });
      ref.read(crateLibraryProvider.notifier).loadCrates();
    } on Exception catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to remove track: $e')),
        );
      }
    }
  }

  Future<void> _showAddFromSetlistDialog(BuildContext context) async {
    final libraryState = ref.read(setlistLibraryProvider);
    if (libraryState.setlists.isEmpty) {
      await ref.read(setlistLibraryProvider.notifier).loadSetlists();
      if (!mounted) return;
      // ignore: use_build_context_synchronously
      await _showAddFromSetlistDialog(context);
      return;
    }

    showDialog<void>(
      context: context,
      builder: (ctx) {
        final setlists = ref.read(setlistLibraryProvider).setlists;
        return AlertDialog(
          title: const Text('Add from Setlist'),
          content: SizedBox(
            width: double.maxFinite,
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: setlists.length,
              itemBuilder: (_, i) {
                final s = setlists[i];
                return ListTile(
                  title: Text(s.name ?? s.prompt,
                      maxLines: 1, overflow: TextOverflow.ellipsis),
                  subtitle: Text('${s.trackCount} tracks'),
                  onTap: () {
                    Navigator.of(ctx).pop();
                    ref
                        .read(crateLibraryProvider.notifier)
                        .addSetlistToCrate(widget.crateId, s.id)
                        .then((_) => _load());
                  },
                );
              },
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
          ],
        );
      },
    );
  }
}
