'use client';

import { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import {
  Loader2,
  Trash2,
  Copy,
  AlertCircle,
  Check,
  Pencil,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { SetlistTrackTile } from '@/components/setlist-track-tile';
import { TransportControls } from '@/components/transport-controls';
import { RefinementChat } from '@/components/refinement-chat';
import { VersionHistory } from '@/components/version-history';
import {
  useSetlist,
  useDeleteSetlist,
  useDuplicateSetlist,
  useUpdateSetlist,
} from '@/hooks/use-setlist';
import { usePlaybackStore } from '@/stores/playback-store';
import { usePrefetchPreviews } from '@/hooks/use-prefetch-previews';

export default function SetlistDetailPage() {
  const params = useParams<{ id: string }>();
  const router = useRouter();
  const id = params.id;

  const { data: setlist, isLoading, error } = useSetlist(id);
  const deleteMutation = useDeleteSetlist();
  const duplicateMutation = useDuplicateSetlist();
  const updateMutation = useUpdateSetlist();
  const { playIndex, reset: resetPlayback } = usePlaybackStore();

  const [isEditingName, setIsEditingName] = useState(false);
  const [editName, setEditName] = useState('');

  usePrefetchPreviews(setlist?.tracks);

  useEffect(() => {
    return () => resetPlayback();
  }, [resetPlayback]);

  const handleDelete = async () => {
    if (!confirm('Delete this setlist?')) return;
    try {
      await deleteMutation.mutateAsync(id);
      router.push('/setlists');
    } catch {
      alert('Failed to delete setlist.');
    }
  };

  const handleDuplicate = async () => {
    try {
      const dup = await duplicateMutation.mutateAsync(id);
      router.push(`/setlists/${dup.id}`);
    } catch {
      alert('Failed to duplicate setlist.');
    }
  };

  const handleSaveName = async () => {
    if (!editName.trim()) return;
    await updateMutation.mutateAsync({ id, name: editName.trim() });
    setIsEditingName(false);
  };

  const handlePlay = (index: number) => {
    playIndex(index);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    );
  }

  if (error || !setlist) {
    return (
      <div className="flex flex-col items-center justify-center py-20 gap-4">
        <AlertCircle className="h-12 w-12 text-red-400" />
        <p className="text-muted-foreground">
          {error instanceof Error ? error.message : 'Setlist not found'}
        </p>
        <Button variant="outline" onClick={() => router.push('/setlists')}>
          Back to Library
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          {isEditingName ? (
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSaveName()}
                autoFocus
                className="rounded-md border border-border bg-muted px-3 py-1 text-lg font-bold text-foreground outline-none focus:ring-1 focus:ring-primary"
              />
              <Button
                size="icon-sm"
                onClick={handleSaveName}
                disabled={updateMutation.isPending}
              >
                {updateMutation.isPending ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <Check className="h-3.5 w-3.5" />
                )}
              </Button>
            </div>
          ) : (
            <button
              onClick={() => {
                setEditName(setlist.name ?? '');
                setIsEditingName(true);
              }}
              className="group flex items-center gap-2"
            >
              <h1 className="text-2xl font-bold text-foreground">
                {setlist.name ?? 'Untitled Setlist'}
              </h1>
              <Pencil className="h-4 w-4 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
            </button>
          )}
          <p className="mt-1 text-sm text-muted-foreground">{setlist.prompt}</p>
          <div className="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
            <span>{setlist.track_count} tracks</span>
            {setlist.energy_profile && <span>Energy: {setlist.energy_profile}</span>}
            {setlist.version_number != null && <span>v{setlist.version_number}</span>}
          </div>
        </div>

        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={handleDuplicate}
            disabled={duplicateMutation.isPending}
            title="Duplicate"
          >
            {duplicateMutation.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </Button>
          <Button
            variant="destructive"
            size="icon-sm"
            onClick={handleDelete}
            disabled={deleteMutation.isPending}
            title="Delete"
          >
            {deleteMutation.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Trash2 className="h-4 w-4" />
            )}
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
        </div>
      )}

      {/* Main content: tracks + refinement sidebar */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        {/* Track list */}
        <div className="space-y-2 lg:col-span-2">
          {setlist.tracks.map((track, i) => (
            <SetlistTrackTile
              key={`${track.position}-${track.title}`}
              track={track}
              index={i}
              onPlay={handlePlay}
            />
          ))}
        </div>

        {/* Sidebar: refinement + version history */}
        <div className="space-y-4 lg:col-span-1">
          <div className="h-[400px]">
            <RefinementChat setlistId={id} />
          </div>
          <VersionHistory setlistId={id} />
        </div>
      </div>

      {/* Transport */}
      <div className="sticky bottom-4">
        <TransportControls tracks={setlist.tracks} onPlay={handlePlay} />
      </div>
    </div>
  );
}
