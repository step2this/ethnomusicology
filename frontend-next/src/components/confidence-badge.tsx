'use client';

const config: Record<string, { bg: string; text: string; tooltip: string }> = {
  high: {
    bg: 'bg-green-900/50 text-green-300 border-green-700/50',
    text: 'High',
    tooltip: 'High confidence: likely a real track',
  },
  medium: {
    bg: 'bg-amber-900/50 text-amber-300 border-amber-700/50',
    text: 'Medium',
    tooltip: 'Medium confidence: track may need verification',
  },
  low: {
    bg: 'bg-red-900/50 text-red-300 border-red-700/50',
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
