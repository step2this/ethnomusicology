'use client';

import { useState, useCallback } from 'react';
import Link from 'next/link';
import { ChevronLeft, ChevronRight, ArrowUpDown, ArrowUp, ArrowDown, Download, Library, Loader2, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useTracks } from '@/hooks/use-tracks';
import { CamelotChip } from '@/components/camelot-chip';
import { EnergyBar } from '@/components/energy-bar';

type SortField = 'title' | 'artist' | 'bpm' | 'key' | 'energy' | 'date_added';
type SortOrder = 'asc' | 'desc';

const COLUMNS: { key: SortField; label: string; sortable: boolean }[] = [
  { key: 'title', label: 'Title', sortable: true },
  { key: 'artist', label: 'Artist', sortable: true },
  { key: 'bpm', label: 'BPM', sortable: true },
  { key: 'key', label: 'Key', sortable: true },
  { key: 'energy', label: 'Energy', sortable: true },
  { key: 'date_added', label: 'Date Added', sortable: true },
];

function SortIcon({ field, sort, order }: { field: SortField; sort: SortField; order: SortOrder }) {
  if (sort !== field) return <ArrowUpDown className="size-3 text-muted-foreground/50" />;
  return order === 'asc' ? (
    <ArrowUp className="size-3 text-primary" />
  ) : (
    <ArrowDown className="size-3 text-primary" />
  );
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays === 0) return 'Today';
  if (diffDays === 1) return 'Yesterday';
  if (diffDays < 7) return `${diffDays}d ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`;
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

export default function TracksPage() {
  const [page, setPage] = useState(1);
  const [sort, setSort] = useState<SortField>('date_added');
  const [order, setOrder] = useState<SortOrder>('desc');

  const { data, isLoading, isError, error } = useTracks({
    page,
    perPage: 25,
    sort,
    order,
  });

  const handleSort = useCallback(
    (field: SortField) => {
      if (sort === field) {
        setOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
      } else {
        setSort(field);
        setOrder(field === 'title' || field === 'artist' || field === 'key' ? 'asc' : 'desc');
      }
      setPage(1);
    },
    [sort],
  );

  // Loading state
  if (isLoading && !data) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Loader2 className="size-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // Error state
  if (isError) {
    return (
      <div className="flex h-96 flex-col items-center justify-center gap-4">
        <AlertCircle className="size-12 text-destructive" />
        <p className="text-sm text-muted-foreground">
          {error instanceof Error ? error.message : 'Failed to load tracks.'}
        </p>
      </div>
    );
  }

  const tracks = data?.data ?? [];
  const totalPages = data?.total_pages ?? 1;
  const total = data?.total ?? 0;

  // Empty state
  if (tracks.length === 0 && page === 1) {
    return (
      <div className="flex h-96 flex-col items-center justify-center gap-4">
        <Library className="size-16 text-muted-foreground/40" />
        <h2 className="text-lg font-semibold text-foreground">No tracks imported yet</h2>
        <p className="text-sm text-muted-foreground">
          Import from Spotify to get started.
        </p>
        <Link href="/import/spotify">
          <Button className="gap-2">
            <Download className="size-4" />
            Import from Spotify
          </Button>
        </Link>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-6xl px-6 py-10">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-foreground">
          Track Catalog
          <span className="ml-2 text-sm font-normal text-muted-foreground">
            ({total} tracks)
          </span>
        </h1>
      </div>

      {/* Table */}
      <div className="overflow-x-auto rounded-md border border-border-default">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border-default bg-surface-content">
              {COLUMNS.map((col) => (
                <th
                  key={col.key}
                  className="px-4 py-3 text-left font-medium text-muted-foreground"
                >
                  {col.sortable ? (
                    <button
                      onClick={() => handleSort(col.key)}
                      className="inline-flex items-center gap-1.5 hover:text-foreground"
                    >
                      {col.label}
                      <SortIcon field={col.key} sort={sort} order={order} />
                    </button>
                  ) : (
                    col.label
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {tracks.map((track) => (
              <tr
                key={track.id}
                className="border-b border-border-default last:border-0 hover:bg-surface-raised/50"
              >
                <td className="max-w-[250px] truncate px-4 py-3 font-medium text-foreground">
                  {track.title}
                </td>
                <td className="max-w-[200px] truncate px-4 py-3 text-text-secondary">
                  {track.artist}
                </td>
                <td className="px-4 py-3 font-mono text-text-secondary tabular-nums">
                  {track.bpm != null ? Math.round(track.bpm) : '-'}
                </td>
                <td className="px-4 py-3">
                  <CamelotChip camelotCode={track.key} />
                </td>
                <td className="px-4 py-3">
                  <EnergyBar energy={track.energy} />
                </td>
                <td className="px-4 py-3 text-text-secondary">
                  {formatDate(track.date_added)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Page {page} of {totalPages}
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page <= 1}
            >
              <ChevronLeft className="size-4" />
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              disabled={page >= totalPages}
            >
              Next
              <ChevronRight className="size-4" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
