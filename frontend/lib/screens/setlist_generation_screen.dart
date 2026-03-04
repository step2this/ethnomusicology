import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/audio_provider.dart';
import '../providers/deezer_provider.dart';
import '../providers/refinement_provider.dart';
import '../providers/setlist_provider.dart';
import '../widgets/setlist_input_form.dart';
import '../widgets/setlist_result_view.dart';
import '../widgets/version_history_panel.dart';

class SetlistGenerationScreen extends ConsumerStatefulWidget {
  const SetlistGenerationScreen({super.key});

  @override
  ConsumerState<SetlistGenerationScreen> createState() =>
      _SetlistGenerationScreenState();
}

class _SetlistGenerationScreenState
    extends ConsumerState<SetlistGenerationScreen> {
  final _scaffoldKey = GlobalKey<ScaffoldState>();

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(setlistProvider);
    final refinementState = ref.watch(refinementProvider);

    // Trigger Deezer prefetch and load history when setlist first appears
    ref.listen(setlistProvider, (previous, next) {
      if ((previous?.hasSetlist ?? false) == false && next.hasSetlist) {
        ref.read(previewProvider.notifier).prefetchForSetlist(
              next.setlist!.tracks,
            );
        ref.read(refinementProvider.notifier).loadHistory(next.setlist!.id);
      }
    });

    return Scaffold(
      key: _scaffoldKey,
      appBar: AppBar(
        title: const Text('Create Set'),
        centerTitle: true,
        actions: [
          if (state.hasSetlist) ...[
            IconButton(
              icon: const Icon(Icons.history),
              onPressed: () => _scaffoldKey.currentState?.openEndDrawer(),
              tooltip: 'Version history',
            ),
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () => _resetAll(),
              tooltip: 'New setlist',
            ),
          ],
        ],
      ),
      endDrawer: state.hasSetlist
          ? VersionHistoryPanel(
              versions: refinementState.versionHistory,
              currentVersion: refinementState.currentVersion,
              isLoading: refinementState.isLoadingHistory,
              onRevert: (versionNumber) {
                ref.read(refinementProvider.notifier).revertToVersion(
                      state.setlist!.id,
                      versionNumber,
                    );
                Navigator.of(context).pop();
              },
            )
          : null,
      body: _buildBody(context, state),
    );
  }

  Widget _buildBody(BuildContext context, SetlistState state) {
    if (state.error != null) {
      return _buildError(context, state);
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

  Widget _buildError(BuildContext context, SetlistState state) {
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
            onPressed: () => _resetAll(),
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

  void _resetAll() {
    ref.read(setlistProvider.notifier).reset();
    ref.read(audioPlaybackProvider.notifier).stop();
    ref.read(refinementProvider.notifier).reset();
  }
}
