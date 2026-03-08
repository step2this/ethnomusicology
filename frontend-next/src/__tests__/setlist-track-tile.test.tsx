import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { SetlistTrack } from '@/types';

// Mock audio-service before importing components that use playback store
vi.mock('@/lib/audio-service', () => ({
  default: {
    subscribe: vi.fn(() => vi.fn()),
    setOnTrackEnded: vi.fn(),
    setVolume: vi.fn(),
    play: vi.fn(),
    pause: vi.fn(),
    resume: vi.fn(),
    stop: vi.fn(),
  },
}));

import { SetlistTrackTile } from '@/components/setlist-track-tile';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

const baseTrack: SetlistTrack = {
  position: 1,
  title: 'Strings of Life',
  artist: 'Derrick May',
  bpm: 126,
  key: 'Am',
  camelot_code: '8A',
  energy: 0.75,
  transition_score: null,
  energy_score: null,
  is_catalog: false,
  catalog_track_id: null,
  spotify_id: null,
  spotify_uri: null,
  bpm_warning: null,
  confidence: 'high',
  verification_notes: null,
};

describe('SetlistTrackTile', () => {
  it('renders position number', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('renders title and artist', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Strings of Life')).toBeInTheDocument();
    expect(screen.getByText('Derrick May')).toBeInTheDocument();
  });

  it('shows BPM metadata chip', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('BPM:')).toBeInTheDocument();
    expect(screen.getByText('126')).toBeInTheDocument();
  });

  it('shows Key metadata chip with camelot code', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Key:')).toBeInTheDocument();
    expect(screen.getByText('8A')).toBeInTheDocument();
  });

  it('shows confidence badge', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('High')).toBeInTheDocument();
  });

  it('shows medium confidence badge', () => {
    const track = { ...baseTrack, confidence: 'medium' };
    render(
      <SetlistTrackTile track={track} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Medium')).toBeInTheDocument();
  });

  it('shows BPM warning when present', () => {
    const track = { ...baseTrack, bpm_warning: 'Large BPM jump from previous track' };
    render(
      <SetlistTrackTile track={track} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Large BPM jump from previous track')).toBeInTheDocument();
  });

  it('does not show BPM warning when null', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.queryByText(/BPM jump/)).not.toBeInTheDocument();
  });

  it('play button calls onPlay with index', () => {
    const onPlay = vi.fn();
    render(
      <SetlistTrackTile track={baseTrack} index={3} onPlay={onPlay} />,
      { wrapper: createWrapper() },
    );
    const playBtn = screen.getByRole('button', { name: 'Play' });
    fireEvent.click(playBtn);
    expect(onPlay).toHaveBeenCalledWith(3);
  });

  it('shows Spotify link when spotify_uri present', () => {
    const track = { ...baseTrack, spotify_uri: 'spotify:track:abc123' };
    render(
      <SetlistTrackTile track={track} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    const spotifyLink = screen.getByTitle('Open in Spotify');
    expect(spotifyLink).toHaveAttribute('href', 'https://open.spotify.com/track/abc123');
    expect(spotifyLink).toHaveAttribute('target', '_blank');
  });

  it('does not show Spotify link when spotify_uri is null', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.queryByTitle('Open in Spotify')).not.toBeInTheDocument();
  });

  it('title links to Google search', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    const titleLink = screen.getByText('Strings of Life').closest('a');
    expect(titleLink?.getAttribute('href')).toContain('google.com/search');
    expect(titleLink?.getAttribute('href')).toContain('Strings');
    expect(titleLink).toHaveAttribute('target', '_blank');
  });

  it('shows verification notes when present', () => {
    const track = { ...baseTrack, verification_notes: 'Track confirmed on Discogs' };
    render(
      <SetlistTrackTile track={track} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Track confirmed on Discogs')).toBeInTheDocument();
  });

  it('shows energy metadata chip', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Energy:')).toBeInTheDocument();
    expect(screen.getByText('0.75')).toBeInTheDocument();
  });

  it('shows flow chip when transition_score is present', () => {
    const track = { ...baseTrack, transition_score: 0.8 };
    render(
      <SetlistTrackTile track={track} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Flow:')).toBeInTheDocument();
    expect(screen.getByText('0.8')).toBeInTheDocument();
  });

  it('does not show flow chip when transition_score is null', () => {
    render(
      <SetlistTrackTile track={baseTrack} index={0} onPlay={vi.fn()} />,
      { wrapper: createWrapper() },
    );
    expect(screen.queryByText('Flow:')).not.toBeInTheDocument();
  });
});
