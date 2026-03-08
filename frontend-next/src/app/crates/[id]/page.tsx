'use client';

import { useState, useEffect, useRef } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { ArrowLeft, Trash2, Plus, Loader2, Inbox, Music } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useCrate, useRemoveCrateTrack, useAddSetlistToCrate } from '@/hooks/use-crates';
import { useSetlists } from '@/hooks/use-setlist';
import type { CrateTrack } from '@/types';

function TrackRow({
  track,
  crateId,
}: {
  track: CrateTrack;
  crateId: string;
}) {
  const removeMutation = useRemoveCrateTrack();

  function handleRemove(e: React.MouseEvent) {
    e.stopPropagation();
    if (window.confirm(`Remove "${track.title}" from this crate?`)) {
      removeMutation.mutate(
        { crateId, trackId: track.id },
        { onError: () => alert('Failed to remove track.') },
      );
    }
  }

  return (
    <div className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
      <div className="flex size-8 shrink-0 items-center justify-center rounded bg-muted text-xs font-medium text-muted-foreground">
        {track.position}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-foreground">
          {track.title}
        </p>
        <p className="truncate text-xs text-muted-foreground">{track.artist}</p>
      </div>
      <div className="hidden shrink-0 items-center gap-3 text-xs text-muted-foreground sm:flex">
        {track.bpm && <span>{track.bpm} BPM</span>}
        {track.key && <span>{track.key}</span>}
        {track.setlist_name && (
          <span className="max-w-[120px] truncate" title={track.setlist_name}>
            {track.setlist_name}
          </span>
        )}
      </div>
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={handleRemove}
        disabled={removeMutation.isPending}
        title="Remove track"
      >
        <Trash2 className="size-4" />
      </Button>
    </div>
  );
}

function AddSetlistDialog({
  crateId,
  onClose,
}: {
  crateId: string;
  onClose: () => void;
}) {
  const { data: setlists, isLoading } = useSetlists();
  const addMutation = useAddSetlistToCrate();

  function handleAdd(setlistId: string) {
    addMutation.mutate(
      { crateId, setlistId },
      { onSuccess: () => onClose() },
    );
  }

  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handleKeyDown);
    dialogRef.current?.focus();
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="add-setlist-title"
    >
      <div
        ref={dialogRef}
        tabIndex={-1}
        className="mx-4 w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-lg outline-none"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="add-setlist-title" className="mb-4 text-lg font-semibold text-foreground">
          Add Setlist to Crate
        </h2>
        {isLoading ? (
          <div className="flex justify-center py-8">
            <Loader2 className="size-6 animate-spin text-muted-foreground" />
          </div>
        ) : !setlists || setlists.length === 0 ? (
          <p className="py-8 text-center text-sm text-muted-foreground">
            No setlists available.
          </p>
        ) : (
          <div className="max-h-64 overflow-y-auto">
            {setlists.map((s) => (
              <button
                key={s.id}
                onClick={() => handleAdd(s.id)}
                disabled={addMutation.isPending}
                className="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors hover:bg-muted/50 disabled:opacity-50"
              >
                <Music className="size-4 shrink-0 text-muted-foreground" />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-foreground">
                    {s.name || 'Untitled'}
                  </p>
                  <p className="truncate text-xs text-muted-foreground">
                    {s.track_count} tracks
                  </p>
                </div>
              </button>
            ))}
          </div>
        )}
        <div className="mt-4 flex justify-end">
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
        </div>
      </div>
    </div>
  );
}

export default function CrateDetailPage() {
  const params = useParams<{ id: string }>();
  const router = useRouter();
  const { data: crate, isLoading, error } = useCrate(params.id);
  const [showAddDialog, setShowAddDialog] = useState(false);

  if (isLoading) {
    return (
      <div className="flex h-[50vh] items-center justify-center">
        <Loader2 className="size-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error || !crate) {
    return (
      <div className="flex h-[50vh] flex-col items-center justify-center gap-4 text-muted-foreground">
        <p>Failed to load crate.</p>
        <Button variant="outline" onClick={() => router.push('/crates')}>
          Back to Crates
        </Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="mb-6">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => router.push('/crates')}
          className="mb-4"
        >
          <ArrowLeft className="size-4" data-icon="inline-start" />
          Back to Crates
        </Button>
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-foreground">{crate.name}</h1>
            <p className="text-sm text-muted-foreground">
              {crate.tracks.length} tracks
            </p>
          </div>
          <Button onClick={() => setShowAddDialog(true)}>
            <Plus className="size-4" data-icon="inline-start" />
            Add Setlist
          </Button>
        </div>
      </div>

      {crate.tracks.length === 0 ? (
        <div className="flex h-[40vh] flex-col items-center justify-center gap-4 text-muted-foreground">
          <Inbox className="size-16 opacity-50" />
          <p className="text-center">
            No tracks in this crate.
            <br />
            Add a setlist to populate it.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {crate.tracks.map((track) => (
            <TrackRow key={track.id} track={track} crateId={crate.id} />
          ))}
        </div>
      )}

      {showAddDialog && (
        <AddSetlistDialog
          crateId={crate.id}
          onClose={() => setShowAddDialog(false)}
        />
      )}
    </div>
  );
}
