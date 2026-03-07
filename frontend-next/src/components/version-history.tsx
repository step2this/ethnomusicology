'use client';

import { useState } from 'react';
import { History, ChevronDown, ChevronUp, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useSetlistHistory, useRevertSetlist } from '@/hooks/use-refinement';

function formatTimestamp(raw: string): string {
  const dt = new Date(raw);
  if (isNaN(dt.getTime())) return raw;
  const now = new Date();
  const diffMs = now.getTime() - dt.getTime();
  const diffMin = Math.floor(diffMs / 60_000);
  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${dt.getMonth() + 1}/${dt.getDate()} ${String(dt.getHours()).padStart(2, '0')}:${String(dt.getMinutes()).padStart(2, '0')}`;
}

export function VersionHistory({ setlistId }: { setlistId: string }) {
  const [expanded, setExpanded] = useState(false);
  const { data: history, isLoading } = useSetlistHistory(setlistId);
  const revertMutation = useRevertSetlist();

  const versions = history?.versions ?? [];

  return (
    <div className="rounded-lg border border-border bg-card">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center gap-2 px-4 py-2.5 text-sm font-medium text-foreground hover:bg-muted/50 transition-colors"
      >
        <History className="h-4 w-4 text-muted-foreground" />
        <span>Version History</span>
        <span className="ml-auto">
          {expanded ? (
            <ChevronUp className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          )}
        </span>
      </button>

      {expanded && (
        <div className="border-t border-border">
          {isLoading && (
            <div className="flex items-center justify-center py-6">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          )}

          {!isLoading && versions.length === 0 && (
            <p className="py-6 text-center text-sm text-muted-foreground">
              No version history yet
            </p>
          )}

          {!isLoading && versions.length > 0 && (
            <div className="divide-y divide-border">
              {versions.map((version) => {
                const isCurrent =
                  version.version_number === versions[versions.length - 1]?.version_number;
                return (
                  <div
                    key={version.version_number}
                    className="flex items-center gap-3 px-4 py-2.5"
                  >
                    <div
                      className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-medium ${
                        isCurrent
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted text-muted-foreground'
                      }`}
                    >
                      v{version.version_number}
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm text-foreground">
                        {version.summary ?? version.action_type ?? 'Initial version'}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {formatTimestamp(version.created_at)} &middot; {version.track_count} tracks
                      </p>
                    </div>
                    {isCurrent ? (
                      <span className="shrink-0 rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
                        Current
                      </span>
                    ) : (
                      <Button
                        variant="ghost"
                        size="xs"
                        onClick={() =>
                          revertMutation.mutate({
                            setlistId,
                            versionNumber: version.version_number,
                          })
                        }
                        disabled={revertMutation.isPending}
                      >
                        Revert
                      </Button>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
