'use client';

const config: Record<string, { bg: string; text: string; tooltip: string }> = {
  high: {
    bg: 'bg-[var(--status-success)]/15 text-[var(--status-success)] border-[var(--status-success)]/30',
    text: 'High',
    tooltip: 'High confidence: likely a real track',
  },
  medium: {
    bg: 'bg-[var(--status-warning)]/15 text-[var(--status-warning)] border-[var(--status-warning)]/30',
    text: 'Medium',
    tooltip: 'Medium confidence: track may need verification',
  },
  low: {
    bg: 'bg-[var(--status-error)]/15 text-[var(--status-error)] border-[var(--status-error)]/30',
    text: 'Low',
    tooltip: 'Low confidence: track may not exist or details may be inaccurate',
  },
};

export function ConfidenceBadge({ confidence }: { confidence: string | null }) {
  if (!confidence || !config[confidence]) return null;

  const { bg, text, tooltip } = config[confidence];

  return (
    <span
      className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${bg}`}
      title={tooltip}
    >
      {text}
    </span>
  );
}
