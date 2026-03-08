import { create } from 'zustand';
import audioService from '@/lib/audio-service';

export type PlaybackStatus = 'idle' | 'loading' | 'playing' | 'paused' | 'error';

interface PlaybackState {
  status: PlaybackStatus;
  currentTrackIndex: number | null;
  trackCount: number;
  volume: number;
  previewUrls: Map<number, string>;

  setStatus: (status: PlaybackStatus) => void;
  setCurrentTrack: (index: number | null) => void;
  setTrackCount: (count: number) => void;
  setPreviewUrl: (index: number, url: string) => void;
  setVolume: (volume: number) => void;
  playIndex: (index: number) => void;
  pause: () => void;
  resume: () => void;
  stop: () => void;
  next: () => void;
  prev: () => void;
  reset: () => void;
}

export const usePlaybackStore = create<PlaybackState>((set, get) => {
  // Sync audio service state into the store
  audioService.subscribe(({ playing }) => {
    set({ status: playing ? 'playing' : (get().currentTrackIndex !== null ? 'paused' : 'idle') });
  });

  // Auto-advance on track end
  audioService.setOnTrackEnded(() => {
    get().next();
  });

  return {
    status: 'idle',
    currentTrackIndex: null,
    trackCount: 0,
    volume: 0.8,
    previewUrls: new Map(),

    setStatus: (status) => set({ status }),
    setCurrentTrack: (index) => set({ currentTrackIndex: index }),
    setTrackCount: (count) => set({ trackCount: count }),

    setPreviewUrl: (index, url) => {
      const map = new Map(get().previewUrls);
      map.set(index, url);
      set({ previewUrls: map });
    },

    setVolume: (volume) => {
      audioService.setVolume(volume);
      set({ volume });
    },

    playIndex: (index) => {
      const { previewUrls, volume } = get();
      const url = previewUrls.get(index);
      if (!url) return;
      set({ currentTrackIndex: index, status: 'loading' });
      audioService.setVolume(volume);
      audioService.play(url).catch(() => {
        set({ status: 'error' });
      });
    },

    pause: () => {
      audioService.pause();
    },

    resume: () => {
      audioService.resume();
    },

    stop: () => {
      audioService.stop();
      set({ currentTrackIndex: null });
    },

    next: () => {
      const { currentTrackIndex, trackCount } = get();
      if (currentTrackIndex === null) return;
      const nextIndex = currentTrackIndex + 1;
      if (nextIndex < trackCount) {
        get().playIndex(nextIndex);
      } else {
        get().stop();
      }
    },

    prev: () => {
      const { currentTrackIndex } = get();
      if (currentTrackIndex === null) return;
      get().playIndex(Math.max(0, currentTrackIndex - 1));
    },

    reset: () => {
      audioService.stop();
      set({ status: 'idle', currentTrackIndex: null, previewUrls: new Map() });
    },
  };
});
