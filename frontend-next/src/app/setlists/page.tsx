'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { Music, Plus, Trash2, Copy, Edit2, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  useSetlists,
  useDeleteSetlist,
  useUpdateSetlist,
  useDuplicateSetlist,
} from '@/hooks/use-setlist';
import type { SetlistSummary } from '@/types';

function formatDate(iso: string): string {
  try {
    const dt = new Date(iso);
    return dt.toLocaleDateString();
  } catch {
    return iso;
  }
}

function SetlistItem({ setlist }: { setlist: SetlistSummary }) {
  const router = useRouter();
  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState(setlist.name ?? setlist.prompt);

  const deleteMutation = useDeleteSetlist();
  const updateMutation = useUpdateSetlist();
  const duplicateMutation = useDuplicateSetlist();

  const displayName = setlist.name || 'Untitled';

  function handleDelete(e: React.MouseEvent) {
    e.stopPropagation();
    if (window.confirm(`Delete "${displayName}"? This cannot be undone.`)) {
      deleteMutation.mutate(setlist.id);
    }
  }

  function handleDuplicate(e: React.MouseEvent) {
    e.stopPropagation();
    duplicateMutation.mutate(setlist.id);
  }

  function handleRename(e: React.MouseEvent) {
    e.stopPropagation();
    setEditName(setlist.name ?? setlist.prompt);
    setIsEditing(true);
  }

  function handleRenameSubmit(e: React.FormEvent) {
    e.preventDefault();
    e.stopPropagation();
    const trimmed = editName.trim();
    if (trimmed) {
      updateMutation.mutate({ id: setlist.id, name: trimmed });
    }
    setIsEditing(false);
  }

  function handleRenameKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation();
      setIsEditing(false);
    }
  }

  return (
    <div
      onClick={() => router.push(`/setlists/${setlist.id}`)}
      className="flex items-center gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-muted/50 cursor-pointer"
    >
      <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
        <Music className="size-5" />
      </div>
      <div className="min-w-0 flex-1">
        {isEditing ? (
          <form onSubmit={handleRenameSubmit} onClick={(e) => e.stopPropagation()}>
            <input
              autoFocus
              type="text"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              onBlur={() => setIsEditing(false)}
              onKeyDown={handleRenameKeyDown}
              className="w-full rounded border border-border bg-background px-2 py-1 text-sm text-foreground outline-none focus:border-ring"
            />
          </form>
        ) : (
          <p className="truncate text-sm font-medium text-foreground">
            {displayName}
          </p>
        )}
        <p className="truncate text-xs text-muted-foreground">{setlist.prompt}</p>
        <p className="text-xs text-muted-foreground">
          {setlist.track_count} tracks · {formatDate(setlist.created_at)}
        </p>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={handleRename}
          title="Rename"
        >
          <Edit2 className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={handleDuplicate}
          disabled={duplicateMutation.isPending}
          title="Duplicate"
        >
          <Copy className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={handleDelete}
          disabled={deleteMutation.isPending}
          title="Delete"
        >
          <Trash2 className="size-4" />
        </Button>
      </div>
    </div>
  );
}

export default function SetlistLibraryPage() {
  const { data: setlists, isLoading, error } = useSetlists();

  if (isLoading) {
    return (
      <div className="flex h-[50vh] items-center justify-center">
        <Loader2 className="size-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-[50vh] flex-col items-center justify-center gap-4 text-muted-foreground">
        <p>Failed to load setlists.</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-foreground">My Setlists</h1>
        <Link href="/setlist/generate">
          <Button>
            <Plus className="size-4" data-icon="inline-start" />
            Generate New
          </Button>
        </Link>
      </div>

      {!setlists || setlists.length === 0 ? (
        <div className="flex h-[40vh] flex-col items-center justify-center gap-4 text-muted-foreground">
          <Music className="size-16 opacity-50" />
          <p className="text-center">
            No saved setlists.
            <br />
            Generate one to get started.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {setlists.map((s) => (
            <SetlistItem key={s.id} setlist={s} />
          ))}
        </div>
      )}
    </div>
  );
}
