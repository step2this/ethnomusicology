'use client';

import { SkipBack, Play, Pause, SkipForward, Square, Volume2 } from 'lucide-react';
import type { SetlistTrack } from '@/types';
import { usePlaybackStore } from '@/stores/playback-store';

export function TransportControls({
  tracks,
  onPlay,
}: {
  tracks: SetlistTrack[];
  onPlay: (index: number) => void;
}) {
  const { status, currentTrackIndex, volume, pause, resume, stop, next, prev, setVolume } =
    usePlaybackStore();

  const playing = status === 'playing';
  const currentTrack = currentTrackIndex !== null ? tracks[currentTrackIndex] : null;

  const handlePlayPause = () => {
    if (playing) {
      pause();
    } else if (currentTrackIndex !== null) {
      resume();
    } else if (tracks.length > 0) {
      onPlay(0);
    }
  };

  return (
    <div className="flex items-center gap-3 rounded-lg border border-secondary bg-secondary/50 px-4 py-2">
      {/* Track info */}
      <div className="mr-2 min-w-0 flex-1">
        {currentTrack ? (
          <div className="truncate">
            <span className="text-sm font-medium text-primary">
              {currentTrack.title}
            </span>
            <span className="text-sm text-muted-foreground">
              {' '}&mdash; {currentTrack.artist}
            </span>
          </div>
        ) : (
          <span className="text-sm text-muted-foreground">No track selected</span>
        )}
      </div>

      {/* Controls */}
      <div className="flex items-center gap-1">
        <button
          onClick={prev}
          disabled={currentTrackIndex === null || currentTrackIndex === 0}
          className="rounded p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30"
          aria-label="Previous track"
        >
          <SkipBack className="h-4 w-4" />
        </button>

        <button
          onClick={handlePlayPause}
          className="rounded-full bg-primary p-2 text-primary-foreground hover:bg-primary/80"
          aria-label={playing ? 'Pause' : 'Play'}
        >
          {playing ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
        </button>

        <button
          onClick={next}
          disabled={currentTrackIndex === null || currentTrackIndex >= tracks.length - 1}
          className="rounded p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30"
          aria-label="Next track"
        >
          <SkipForward className="h-4 w-4" />
        </button>

        <button
          onClick={stop}
          disabled={currentTrackIndex === null}
          className="rounded p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30"
          aria-label="Stop"
        >
          <Square className="h-4 w-4" />
        </button>
      </div>

      {/* Volume */}
      <div className="flex items-center gap-2">
        <Volume2 className="h-4 w-4 text-muted-foreground" />
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={volume}
          onChange={(e) => setVolume(parseFloat(e.target.value))}
          className="h-1 w-20 cursor-pointer accent-primary"
          aria-label="Volume"
        />
      </div>
    </div>
  );
}
