'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import {
  Loader2,
  Sparkles,
  Shuffle,
  Save,
  AlertCircle,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { SetlistTrackTile } from '@/components/setlist-track-tile';
import { TransportControls } from '@/components/transport-controls';
import { useGenerateSetlist, useArrangeSetlist, useUpdateSetlist } from '@/hooks/use-setlist';
import { useGenerationStore } from '@/stores/generation-store';
import { usePlaybackStore } from '@/stores/playback-store';
import { usePrefetchPreviews } from '@/hooks/use-prefetch-previews';
import type { Setlist } from '@/types';

const energyProfiles = [
  { value: 'warm-up', label: 'Warm-Up' },
  { value: 'peak-time', label: 'Peak-Time' },
  { value: 'journey', label: 'Journey' },
  { value: 'steady', label: 'Steady' },
];

export default function GenerateSetlistPage() {
  const router = useRouter();
  const generateMutation = useGenerateSetlist();
  const arrangeMutation = useArrangeSetlist();
  const updateMutation = useUpdateSetlist();
  const { isGenerating, isArranging, error, setGenerating, setArranging, setError, reset: resetGeneration } = useGenerationStore();
  const { playIndex, reset: resetPlayback } = usePlaybackStore();

  const [prompt, setPrompt] = useState('');
  const [trackCount, setTrackCountLocal] = useState(12);
  const [energyProfile, setEnergyProfile] = useState<string | null>(null);
  const [bpmMin, setBpmMin] = useState('');
  const [bpmMax, setBpmMax] = useState('');
  const [creativeMode, setCreativeMode] = useState(false);
  const [verify, setVerify] = useState(true);
  const [setlist, setSetlist] = useState<Setlist | null>(null);
  const [saveName, setSaveName] = useState('');

  usePrefetchPreviews(setlist?.tracks);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      resetPlayback();
      resetGeneration();
    };
  }, [resetPlayback, resetGeneration]);

  const handleGenerate = async () => {
    if (!prompt.trim()) return;
    setError(null);
    setGenerating(true);
    setSetlist(null);
    resetPlayback();

    try {
      const result = await generateMutation.mutateAsync({
        prompt: prompt.trim(),
        trackCount,
        energyProfile: energyProfile ?? undefined,
        creativeMode: creativeMode || undefined,
        bpmMin: bpmMin ? Number(bpmMin) : undefined,
        bpmMax: bpmMax ? Number(bpmMax) : undefined,
        verify: verify || undefined,
      });
      setSetlist(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Generation failed');
    } finally {
      setGenerating(false);
    }
  };

  const handleArrange = async () => {
    if (!setlist) return;
    setArranging(true);
    try {
      const result = await arrangeMutation.mutateAsync({
        id: setlist.id,
        energyProfile: energyProfile ?? undefined,
      });
      setSetlist(result);
      resetPlayback();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Arrange failed');
    } finally {
      setArranging(false);
    }
  };

  const handleSave = async () => {
    if (!setlist || !saveName.trim()) return;
    try {
      await updateMutation.mutateAsync({ id: setlist.id, name: saveName.trim() });
      router.push(`/setlists/${setlist.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Save failed');
    }
  };

  const handlePlay = (index: number) => {
    playIndex(index);
  };

  // --- Form view ---
  if (!setlist) {
    return (
      <div className="mx-auto max-w-2xl space-y-6">
        <h1 className="text-2xl font-bold text-foreground">Generate Setlist</h1>

        {/* Prompt */}
        <div>
          <label className="mb-1.5 block text-sm font-medium text-foreground">
            Describe the vibe
          </label>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Deep progressive house for a late-night rooftop set..."
            rows={3}
            className="w-full rounded-lg border border-border bg-muted px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-primary resize-none"
          />
        </div>

        {/* Energy profile */}
        <div>
          <label className="mb-1.5 block text-sm font-medium text-foreground">
            Energy Profile
          </label>
          <div className="flex flex-wrap gap-2">
            {energyProfiles.map((ep) => (
              <button
                key={ep.value}
                onClick={() =>
                  setEnergyProfile(energyProfile === ep.value ? null : ep.value)
                }
                className={`rounded-full border px-3 py-1 text-sm transition-colors ${
                  energyProfile === ep.value
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border text-muted-foreground hover:text-foreground hover:border-foreground/30'
                }`}
              >
                {ep.label}
              </button>
            ))}
          </div>
        </div>

        {/* Track count */}
        <div>
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium text-foreground">Set Length</label>
            <span className="text-sm text-muted-foreground">{trackCount} tracks</span>
          </div>
          <input
            type="range"
            min={8}
            max={20}
            value={trackCount}
            onChange={(e) => setTrackCountLocal(Number(e.target.value))}
            className="mt-1 w-full accent-primary"
          />
        </div>

        {/* Toggles */}
        <div className="flex flex-col gap-3">
          <label className="flex items-center gap-2 text-sm text-foreground cursor-pointer">
            <input
              type="checkbox"
              checked={creativeMode}
              onChange={(e) => setCreativeMode(e.target.checked)}
              className="rounded accent-primary"
            />
            Creative mode — unexpected but compatible combinations
          </label>
          <label className="flex items-center gap-2 text-sm text-foreground cursor-pointer">
            <input
              type="checkbox"
              checked={verify}
              onChange={(e) => setVerify(e.target.checked)}
              className="rounded accent-primary"
            />
            Verify tracks — double-check with MusicBrainz (~15s)
          </label>
        </div>

        {/* BPM range */}
        <div className="flex items-center gap-3">
          <div className="flex-1">
            <label className="mb-1 block text-sm font-medium text-foreground">
              Min BPM
            </label>
            <input
              type="number"
              value={bpmMin}
              onChange={(e) => setBpmMin(e.target.value)}
              placeholder="e.g. 120"
              className="w-full rounded-md border border-border bg-muted px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          <div className="flex-1">
            <label className="mb-1 block text-sm font-medium text-foreground">
              Max BPM
            </label>
            <input
              type="number"
              value={bpmMax}
              onChange={(e) => setBpmMax(e.target.value)}
              placeholder="e.g. 135"
              className="w-full rounded-md border border-border bg-muted px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="flex items-center gap-2 rounded-lg border border-red-700/50 bg-red-900/30 px-4 py-3 text-sm text-red-300">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {/* Submit */}
        <Button
          onClick={handleGenerate}
          disabled={!prompt.trim() || isGenerating}
          className="w-full"
          size="lg"
        >
          {isGenerating ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              Generating...
            </>
          ) : (
            <>
              <Sparkles className="h-4 w-4" />
              Generate Setlist
            </>
          )}
        </Button>
      </div>
    );
  }

  // --- Result view ---
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">
            {setlist.name ?? 'Generated Setlist'}
          </h1>
          <p className="text-sm text-muted-foreground">{setlist.prompt}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleArrange}
            disabled={isArranging}
          >
            {isArranging ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Shuffle className="h-4 w-4" />
            )}
            Arrange
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              resetPlayback();
              resetGeneration();
              setSetlist(null);
            }}
          >
            New
          </Button>
        </div>
      </div>

      {/* Score */}
      {setlist.score_breakdown?.total != null && (
        <div className="flex items-center gap-4 text-sm">
          <span className="font-medium text-primary">
            Score: {setlist.score_breakdown.total}
          </span>
          {setlist.score_breakdown.harmonic_flow != null && (
            <span className="text-muted-foreground">
              Harmonic: {setlist.score_breakdown.harmonic_flow}
            </span>
          )}
          {setlist.score_breakdown.energy_arc != null && (
            <span className="text-muted-foreground">
              Energy: {setlist.score_breakdown.energy_arc}
            </span>
          )}
          {setlist.score_breakdown.bpm_consistency != null && (
            <span className="text-muted-foreground">
              BPM: {setlist.score_breakdown.bpm_consistency}
            </span>
          )}
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-red-700/50 bg-red-900/30 px-4 py-3 text-sm text-red-300">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Track list */}
      <div className="space-y-2">
        {setlist.tracks.map((track, i) => (
          <SetlistTrackTile
            key={`${track.position}-${track.title}`}
            track={track}
            index={i}
            onPlay={handlePlay}
          />
        ))}
      </div>

      {/* Save bar */}
      <div className="flex items-center gap-2 rounded-lg border border-border bg-card p-3">
        <input
          type="text"
          value={saveName}
          onChange={(e) => setSaveName(e.target.value)}
          placeholder="Name your setlist..."
          className="flex-1 rounded-md bg-muted px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-primary"
        />
        <Button
          onClick={handleSave}
          disabled={!saveName.trim() || updateMutation.isPending}
          size="sm"
        >
          {updateMutation.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Save className="h-4 w-4" />
          )}
          Save
        </Button>
      </div>

      {/* Transport */}
      <div className="sticky bottom-4">
        <TransportControls tracks={setlist.tracks} onPlay={handlePlay} />
      </div>
    </div>
  );
}
