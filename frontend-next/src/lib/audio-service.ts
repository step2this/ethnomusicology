type StateListener = (state: { playing: boolean; currentUrl: string | null }) => void;

class AudioService {
  private audio: HTMLAudioElement | null = null;
  private onTrackEnded: (() => void) | null = null;
  private listeners: Set<StateListener> = new Set();
  private currentUrl: string | null = null;
  private playing = false;

  private emit() {
    const state = { playing: this.playing, currentUrl: this.currentUrl };
    for (const listener of this.listeners) {
      listener(state);
    }
  }

  private ensureAudio(): HTMLAudioElement {
    if (!this.audio) {
      this.audio = new Audio();
      this.audio.addEventListener('ended', () => {
        this.playing = false;
        this.emit();
        this.onTrackEnded?.();
      });
      this.audio.addEventListener('error', () => {
        this.playing = false;
        this.emit();
      });
    }
    return this.audio;
  }

  async play(url: string): Promise<void> {
    const audio = this.ensureAudio();
    if (this.currentUrl !== url) {
      audio.src = url;
      this.currentUrl = url;
    }
    await audio.play();
    this.playing = true;
    this.emit();
  }

  pause(): void {
    this.audio?.pause();
    this.playing = false;
    this.emit();
  }

  resume(): void {
    if (this.audio && this.currentUrl) {
      this.audio.play();
      this.playing = true;
      this.emit();
    }
  }

  stop(): void {
    if (this.audio) {
      this.audio.pause();
      this.audio.currentTime = 0;
    }
    this.playing = false;
    this.currentUrl = null;
    this.emit();
  }

  setVolume(v: number): void {
    const clamped = Math.max(0, Math.min(1, v));
    this.ensureAudio().volume = clamped;
  }

  setOnTrackEnded(cb: () => void): void {
    this.onTrackEnded = cb;
  }

  isPlaying(): boolean {
    return this.playing;
  }

  getCurrentUrl(): string | null {
    return this.currentUrl;
  }

  subscribe(listener: StateListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}

const audioService = new AudioService();
export default audioService;
