import 'package:flutter/material.dart';

import '../config/constants.dart';
import '../providers/audio_provider.dart';

class TransportControls extends StatelessWidget {
  final AudioPlaybackState audioState;
  final int trackCount;
  final VoidCallback? onPrevious;
  final VoidCallback? onTogglePause;
  final VoidCallback? onStop;
  final VoidCallback? onNext;
  final ValueChanged<double> onCrossfadeChanged;

  const TransportControls({
    super.key,
    required this.audioState,
    required this.trackCount,
    this.onPrevious,
    this.onTogglePause,
    this.onStop,
    this.onNext,
    required this.onCrossfadeChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      child: Row(
        children: [
          IconButton(
            icon: const Icon(Icons.skip_previous),
            tooltip: 'Previous',
            onPressed: (audioState.currentTrackIndex == null ||
                    audioState.currentTrackIndex == 0)
                ? null
                : onPrevious,
          ),
          IconButton(
            icon: Icon(
              audioState.isPlaying ? Icons.pause : Icons.play_arrow,
            ),
            tooltip: audioState.isPlaying ? 'Pause' : 'Play',
            onPressed: (audioState.isPlaying || audioState.isPaused)
                ? onTogglePause
                : null,
          ),
          IconButton(
            icon: const Icon(Icons.stop),
            tooltip: 'Stop',
            onPressed: (audioState.isPlaying ||
                    audioState.isPaused ||
                    audioState.isLoading)
                ? onStop
                : null,
          ),
          IconButton(
            icon: const Icon(Icons.skip_next),
            tooltip: 'Next',
            onPressed: (audioState.currentTrackIndex == null ||
                    audioState.currentTrackIndex! >= trackCount - 1)
                ? null
                : onNext,
          ),
          const Text('Crossfade:'),
          SizedBox(
            width: 100,
            child: Slider(
              value: audioState.crossfadeDuration,
              min: AppConstants.minCrossfadeDuration,
              max: AppConstants.maxCrossfadeDuration,
              divisions: AppConstants.crossfadeDivisions,
              label: '${audioState.crossfadeDuration.round()}s',
              onChanged: onCrossfadeChanged,
            ),
          ),
          Text('${audioState.crossfadeDuration.round()}s'),
          const Spacer(),
          if (audioState.currentTrackIndex != null)
            Padding(
              padding: const EdgeInsets.only(right: 8),
              child: Text(
                'Track ${audioState.currentTrackIndex! + 1} of ${audioState.totalTracks}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
          if (audioState.status == PlaybackStatus.completed)
            Text(
              'Set complete',
              style: Theme.of(context).textTheme.bodySmall,
            )
          else if (audioState.statusText != null)
            Flexible(
              child: Text(
                audioState.statusText!,
                style: Theme.of(context).textTheme.bodySmall,
                overflow: TextOverflow.ellipsis,
              ),
            ),
        ],
      ),
    );
  }
}
