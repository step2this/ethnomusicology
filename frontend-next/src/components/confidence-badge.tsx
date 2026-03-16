'use client';

const config: Record<string, { bg: string; text: string; tooltip: string }> = {
  high: {
    bg: 'bg-status-success/15 text-status-success border-status-success/30',
    text: 'High',
    tooltip: 'High confidence: likely a real track',
  },
  medium: {
    bg: 'bg-status-warning/15 text-status-warning border-status-warning/30',
    text: 'Medium',
    tooltip: 'Medium confidence: track may need verification',
  },
  low: {
    bg: 'bg-status-error/15 text-status-error border-status-error/30',
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
