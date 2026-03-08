import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock audio-service before importing stores
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

import { usePlaybackStore } from '@/stores/playback-store';
import { useGenerationStore } from '@/stores/generation-store';

describe('playback-store', () => {
  beforeEach(() => {
    usePlaybackStore.getState().reset();
  });

  it('has correct initial state', () => {
    const state = usePlaybackStore.getState();
    expect(state.status).toBe('idle');
    expect(state.currentTrackIndex).toBeNull();
    expect(state.trackCount).toBe(0);
    expect(state.volume).toBe(0.8);
    expect(state.previewUrls.size).toBe(0);
  });

  it('setStatus updates status', () => {
    usePlaybackStore.getState().setStatus('playing');
    expect(usePlaybackStore.getState().status).toBe('playing');
  });

  it('setStatus cycles through all statuses', () => {
    const statuses = ['idle', 'loading', 'playing', 'paused', 'error'] as const;
    for (const s of statuses) {
      usePlaybackStore.getState().setStatus(s);
      expect(usePlaybackStore.getState().status).toBe(s);
    }
  });

  it('setCurrentTrack updates index', () => {
    usePlaybackStore.getState().setCurrentTrack(3);
    expect(usePlaybackStore.getState().currentTrackIndex).toBe(3);
  });

  it('setCurrentTrack accepts null', () => {
    usePlaybackStore.getState().setCurrentTrack(5);
    usePlaybackStore.getState().setCurrentTrack(null);
    expect(usePlaybackStore.getState().currentTrackIndex).toBeNull();
  });

  it('setTrackCount updates count', () => {
    usePlaybackStore.getState().setTrackCount(12);
    expect(usePlaybackStore.getState().trackCount).toBe(12);
  });

  it('setVolume updates volume and calls audioService', async () => {
    const audioService = (await import('@/lib/audio-service')).default;
    usePlaybackStore.getState().setVolume(0.5);
    expect(usePlaybackStore.getState().volume).toBe(0.5);
    expect(audioService.setVolume).toHaveBeenCalledWith(0.5);
  });

  it('setPreviewUrl adds url to map', () => {
    usePlaybackStore.getState().setPreviewUrl(0, 'https://example.com/track.mp3');
    expect(usePlaybackStore.getState().previewUrls.get(0)).toBe('https://example.com/track.mp3');
  });

  it('setPreviewUrl overwrites existing url', () => {
    usePlaybackStore.getState().setPreviewUrl(0, 'https://example.com/old.mp3');
    usePlaybackStore.getState().setPreviewUrl(0, 'https://example.com/new.mp3');
    expect(usePlaybackStore.getState().previewUrls.get(0)).toBe('https://example.com/new.mp3');
  });

  it('reset clears state back to initial', () => {
    usePlaybackStore.getState().setStatus('playing');
    usePlaybackStore.getState().setCurrentTrack(5);
    usePlaybackStore.getState().setPreviewUrl(0, 'https://example.com/track.mp3');
    usePlaybackStore.getState().reset();

    const state = usePlaybackStore.getState();
    expect(state.status).toBe('idle');
    expect(state.currentTrackIndex).toBeNull();
    expect(state.previewUrls.size).toBe(0);
  });
});

describe('generation-store', () => {
  beforeEach(() => {
    useGenerationStore.getState().reset();
  });

  it('has correct initial state', () => {
    const state = useGenerationStore.getState();
    expect(state.isGenerating).toBe(false);
    expect(state.isArranging).toBe(false);
    expect(state.error).toBeNull();
  });

  it('setGenerating updates flag', () => {
    useGenerationStore.getState().setGenerating(true);
    expect(useGenerationStore.getState().isGenerating).toBe(true);
  });

  it('setGenerating can toggle off', () => {
    useGenerationStore.getState().setGenerating(true);
    useGenerationStore.getState().setGenerating(false);
    expect(useGenerationStore.getState().isGenerating).toBe(false);
  });

  it('setArranging updates flag', () => {
    useGenerationStore.getState().setArranging(true);
    expect(useGenerationStore.getState().isArranging).toBe(true);
  });

  it('setArranging can toggle off', () => {
    useGenerationStore.getState().setArranging(true);
    useGenerationStore.getState().setArranging(false);
    expect(useGenerationStore.getState().isArranging).toBe(false);
  });

  it('setError sets error message', () => {
    useGenerationStore.getState().setError('Something went wrong');
    expect(useGenerationStore.getState().error).toBe('Something went wrong');
  });

  it('setError clears with null', () => {
    useGenerationStore.getState().setError('err');
    useGenerationStore.getState().setError(null);
    expect(useGenerationStore.getState().error).toBeNull();
  });

  it('reset clears all state', () => {
    useGenerationStore.getState().setGenerating(true);
    useGenerationStore.getState().setArranging(true);
    useGenerationStore.getState().setError('test');
    useGenerationStore.getState().reset();

    const state = useGenerationStore.getState();
    expect(state.isGenerating).toBe(false);
    expect(state.isArranging).toBe(false);
    expect(state.error).toBeNull();
  });

  it('can set generating and error simultaneously', () => {
    useGenerationStore.getState().setGenerating(true);
    useGenerationStore.getState().setError('timeout');
    const state = useGenerationStore.getState();
    expect(state.isGenerating).toBe(true);
    expect(state.error).toBe('timeout');
  });
});
