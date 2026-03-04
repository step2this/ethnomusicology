import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/audio_provider.dart';
import '../providers/deezer_provider.dart';
import '../providers/setlist_provider.dart';
import '../widgets/setlist_input_form.dart';
import '../widgets/setlist_result_view.dart';

class SetlistGenerationScreen extends ConsumerWidget {
  const SetlistGenerationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(setlistProvider);

    // Trigger Deezer prefetch when setlist is first displayed
    ref.listen(setlistProvider, (previous, next) {
      if ((previous?.hasSetlist ?? false) == false && next.hasSetlist) {
        ref.read(deezerPreviewProvider.notifier).prefetchForSetlist(
              next.setlist!.tracks,
            );
      }
    });

    return Scaffold(
      appBar: AppBar(
        title: const Text('Create Set'),
        centerTitle: true,
        actions: [
          if (state.hasSetlist)
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () => _resetAll(ref),
              tooltip: 'New setlist',
            ),
        ],
      ),
      body: _buildBody(context, state, ref),
    );
  }

  Widget _buildBody(BuildContext context, SetlistState state, WidgetRef ref) {
    if (state.error != null) {
      return _buildError(context, state, ref);
    }
    if (state.isGenerating) {
      return _buildLoading('Generating your setlist with Claude...');
    }
    if (state.isArranging) {
      return _buildLoading('Arranging for harmonic flow...');
    }
    if (state.hasSetlist) {
      return SetlistResultView(
        setlist: state.setlist!,
        onArrange: () => ref.read(setlistProvider.notifier).arrangeSetlist(),
      );
    }
    return SetlistInputForm(
      onGenerate: ({
        required prompt,
        sourcePlaylistId,
        seedTracklist,
        required trackCount,
        energyProfile,
        creativeMode,
        bpmMin,
        bpmMax,
      }) {
        ref.read(setlistProvider.notifier).generateSetlist(
              prompt,
              trackCount: trackCount,
              energyProfile: energyProfile,
              creativeMode: creativeMode,
              sourcePlaylistId: sourcePlaylistId,
              seedTracklist: seedTracklist,
              bpmMin: bpmMin,
              bpmMax: bpmMax,
            );
      },
    );
  }

  Widget _buildError(BuildContext context, SetlistState state, WidgetRef ref) {
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
            onPressed: () => _resetAll(ref),
            child: const Text('Try Again'),
          ),
        ],
      ),
    );
  }

  Widget _buildLoading(String message) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 16),
          Text(message),
        ],
      ),
    );
  }

  void _resetAll(WidgetRef ref) {
    ref.read(setlistProvider.notifier).reset();
    ref.read(audioPlaybackProvider.notifier).stop();
  }
}
