'use client';

import { Play, Pause, AlertTriangle, ExternalLink } from 'lucide-react';
import type { SetlistTrack } from '@/types';
import { usePlaybackStore } from '@/stores/playback-store';
import { MetadataChip } from '@/components/metadata-chip';
import { ConfidenceBadge } from '@/components/confidence-badge';
import { PurchaseLinkPanel } from '@/components/purchase-link-panel';

export function SetlistTrackTile({
  track,
  index,
  onPlay,
}: {
  track: SetlistTrack;
  index: number;
  onPlay: (index: number) => void;
}) {
  const { status, currentTrackIndex, pause } = usePlaybackStore();
  const isThisPlaying = currentTrackIndex === index && status === 'playing';
  const isThisPaused = currentTrackIndex === index && status === 'paused';

  const searchUrl = `https://www.google.com/search?q=${encodeURIComponent(
    `"${track.title}" "${track.artist}"`,
  )}`;

  const spotifyUrl = track.spotify_uri
    ? `https://open.spotify.com/track/${track.spotify_uri.split(':').pop()}`
    : null;

  return (
    <div className="flex items-start gap-3 rounded-lg border border-border bg-card p-3 transition-colors hover:bg-card/80">
      {/* Position */}
      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-primary/10 text-sm font-bold text-primary">
        {track.position}
      </div>

      {/* Main content */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <a
            href={searchUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="truncate font-medium text-foreground underline decoration-foreground/30 hover:decoration-foreground"
          >
            {track.title}
          </a>
          {spotifyUrl && (
            <a
              href={spotifyUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="shrink-0 text-primary hover:text-primary/80"
              title="Open in Spotify"
            >
              <ExternalLink className="h-3.5 w-3.5" />
            </a>
          )}
          <ConfidenceBadge confidence={track.confidence} />
        </div>

        <p className="truncate text-sm text-muted-foreground">{track.artist}</p>

        {track.verification_notes && (
          <p className="mt-0.5 text-xs text-amber-400">{track.verification_notes}</p>
        )}

        {/* Metadata chips */}
        <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
          <MetadataChip label="BPM" value={track.bpm} />
          <MetadataChip label="Key" value={track.camelot_code ?? track.key} />
          <MetadataChip label="Energy" value={track.energy} />
          {track.transition_score != null && (
            <MetadataChip label="Flow" value={track.transition_score} />
          )}
        </div>

        {/* BPM warning */}
        {track.bpm_warning && (
          <div className="mt-1 flex items-center gap-1 text-xs text-amber-400">
            <AlertTriangle className="h-3.5 w-3.5" />
            <span>{track.bpm_warning}</span>
          </div>
        )}

        <PurchaseLinkPanel title={track.title} artist={track.artist} />
      </div>

      {/* Play button */}
      <button
        onClick={() => {
          if (isThisPlaying) {
            pause();
          } else {
            onPlay(index);
          }
        }}
        className="shrink-0 rounded-full bg-primary/10 p-2 text-primary hover:bg-primary/20 transition-colors"
        aria-label={isThisPlaying ? 'Pause' : 'Play'}
      >
        {isThisPlaying ? (
          <Pause className="h-4 w-4" />
        ) : (
          <Play className="h-4 w-4" />
        )}
      </button>
    </div>
  );
}
