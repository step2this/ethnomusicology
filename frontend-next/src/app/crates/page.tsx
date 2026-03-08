'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Inbox, Plus, Trash2, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useCrates, useCreateCrate, useDeleteCrate } from '@/hooks/use-crates';

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleDateString();
  } catch {
    return iso;
  }
}

export default function CrateLibraryPage() {
  const router = useRouter();
  const { data: crates, isLoading, error } = useCrates();
  const createMutation = useCreateCrate();
  const deleteMutation = useDeleteCrate();
  const [newName, setNewName] = useState('');

  function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = newName.trim();
    if (trimmed) {
      createMutation.mutate(trimmed, {
        onError: () => alert('Failed to create crate.'),
      });
      setNewName('');
    }
  }

  function handleDelete(e: React.MouseEvent, id: string, name: string) {
    e.stopPropagation();
    if (window.confirm(`Delete "${name}"? This cannot be undone.`)) {
      deleteMutation.mutate(id, {
        onError: () => alert('Failed to delete crate.'),
      });
    }
  }

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
        <p>Failed to load crates.</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <h1 className="mb-6 text-2xl font-bold text-foreground">My Crates</h1>

      <form onSubmit={handleCreate} className="mb-6 flex gap-2">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="New crate name..."
          className="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:border-ring"
        />
        <Button type="submit" disabled={!newName.trim() || createMutation.isPending}>
          <Plus className="size-4" data-icon="inline-start" />
          Create
        </Button>
      </form>

      {!crates || crates.length === 0 ? (
        <div className="flex h-[40vh] flex-col items-center justify-center gap-4 text-muted-foreground">
          <Inbox className="size-16 opacity-50" />
          <p className="text-center">
            No crates yet.
            <br />
            Create one to organize your setlists.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {crates.map((c) => (
            <div
              key={c.id}
              role="button"
              tabIndex={0}
              onClick={() => router.push(`/crates/${c.id}`)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); router.push(`/crates/${c.id}`); } }}
              className="flex items-center gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-muted/50 cursor-pointer"
            >
              <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                <Inbox className="size-5" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium text-foreground">
                  {c.name}
                </p>
                <p className="text-xs text-muted-foreground">
                  {c.track_count} tracks · Updated {formatDate(c.updated_at)}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={(e) => handleDelete(e, c.id, c.name)}
                disabled={deleteMutation.isPending}
                title="Delete crate"
              >
                <Trash2 className="size-4" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
