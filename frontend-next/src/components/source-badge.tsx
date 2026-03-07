'use client';

const sourceColors: Record<string, string> = {
  deezer: 'bg-purple-500',
  itunes: 'bg-pink-500',
  soundcloud: 'bg-orange-500',
};

export function SourceBadge({ source }: { source: string | null }) {
  if (!source) return null;

  const dotColor = sourceColors[source.toLowerCase()] ?? 'bg-gray-500';

  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
      <span className={`inline-block h-2 w-2 rounded-full ${dotColor}`} />
      {source}
    </span>
  );
}
