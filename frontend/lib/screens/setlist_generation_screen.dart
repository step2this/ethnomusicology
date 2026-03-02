import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/setlist_provider.dart';
import '../widgets/setlist_track_tile.dart';

class SetlistGenerationScreen extends ConsumerStatefulWidget {
  const SetlistGenerationScreen({super.key});

  @override
  ConsumerState<SetlistGenerationScreen> createState() =>
      _SetlistGenerationScreenState();
}

class _SetlistGenerationScreenState
    extends ConsumerState<SetlistGenerationScreen> {
  final _promptController = TextEditingController();

  @override
  void dispose() {
    _promptController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(setlistProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Generate Setlist'),
        centerTitle: true,
        actions: [
          if (state.hasSetlist)
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () {
                ref.read(setlistProvider.notifier).reset();
                _promptController.clear();
              },
              tooltip: 'New setlist',
            ),
        ],
      ),
      body: Column(
        children: [
          // Prompt input area
          _buildPromptInput(state),
          // Content area
          Expanded(child: _buildContent(state)),
        ],
      ),
    );
  }

  Widget _buildPromptInput(SetlistState state) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: _promptController,
              decoration: const InputDecoration(
                hintText: 'Describe your ideal setlist...',
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.music_note),
              ),
              maxLines: 2,
              minLines: 1,
              enabled: !state.isLoading,
              textInputAction: TextInputAction.send,
              onSubmitted: (_) => _generate(),
            ),
          ),
          const SizedBox(width: 8),
          FilledButton(
            onPressed: state.isLoading ? null : _generate,
            child: state.isGenerating
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('Generate'),
          ),
        ],
      ),
    );
  }

  Widget _buildContent(SetlistState state) {
    // Error state
    if (state.error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.error_outline,
                size: 48, color: Theme.of(context).colorScheme.error),
            const SizedBox(height: 16),
            Text(state.error!, textAlign: TextAlign.center),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: () => ref.read(setlistProvider.notifier).reset(),
              child: const Text('Try Again'),
            ),
          ],
        ),
      );
    }

    // Loading state
    if (state.isGenerating) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Generating your setlist...'),
          ],
        ),
      );
    }

    if (state.isArranging) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Arranging for harmonic flow...'),
          ],
        ),
      );
    }

    // Setlist result
    if (state.hasSetlist) {
      return _buildSetlistResult(state);
    }

    // Idle state — prompt guidance
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.queue_music,
                size: 64, color: Theme.of(context).colorScheme.primary),
            const SizedBox(height: 16),
            Text(
              'Describe your ideal set',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            Text(
              'Try something like "Deep house warm-up set for a rooftop party" or "High energy Afrobeats mix for a wedding reception"',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSetlistResult(SetlistState state) {
    final setlist = state.setlist!;

    return Column(
      children: [
        // Header with notes and arrange button
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
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              Text(
                '${setlist.tracks.length} tracks',
                style: Theme.of(context).textTheme.labelLarge,
              ),
              if (setlist.isArranged) ...[
                const SizedBox(width: 12),
                Chip(
                  label: Text(
                      'Flow: ${setlist.harmonicFlowScoreFormatted}'),
                  avatar: const Icon(Icons.auto_awesome, size: 16),
                ),
              ],
              const Spacer(),
              if (!setlist.isArranged)
                FilledButton.icon(
                  onPressed: () =>
                      ref.read(setlistProvider.notifier).arrangeSetlist(),
                  icon: const Icon(Icons.auto_awesome),
                  label: const Text('Arrange'),
                ),
            ],
          ),
        ),
        // Track list
        Expanded(
          child: ListView.builder(
            itemCount: setlist.tracks.length,
            itemBuilder: (context, index) {
              return SetlistTrackTile(track: setlist.tracks[index]);
            },
          ),
        ),
      ],
    );
  }

  void _generate() {
    final prompt = _promptController.text.trim();
    if (prompt.isEmpty) return;
    ref.read(setlistProvider.notifier).generateSetlist(prompt);
  }
}
