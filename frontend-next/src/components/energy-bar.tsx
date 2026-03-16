'use client';

const TOTAL_SEGMENTS = 5;
const SEGMENT_HEIGHTS = [4, 7, 10, 12, 14]; // Staggered heights in px

function getSegmentColor(segIndex: number, filledCount: number): string {
  if (segIndex >= filledCount) return 'var(--border-default, #1E1A16)';
  // Map filled segments: low (blue) → mid (amber) → high (red)
  const ratio = filledCount <= 1 ? 0 : segIndex / (filledCount - 1);
  if (ratio < 0.4) return 'var(--energy-low, #5A9EC0)';
  if (ratio < 0.7) return 'var(--energy-mid, #E8963A)';
  return 'var(--energy-high, #D05040)';
}

export function EnergyBar({ energy }: { energy: number | null | undefined }) {
  if (energy == null) {
    return <span className="text-xs text-muted-foreground">-</span>;
  }

  // energy is 0-1 (percentage) or 0-10
  const normalized = energy > 1 ? energy / 10 : energy;
  const filledCount = Math.round(normalized * TOTAL_SEGMENTS);

  return (
    <div
      className="inline-flex items-end gap-[2px]"
      title={`Energy: ${Math.round(normalized * 100)}%`}
      role="img"
      aria-label={`Energy: ${Math.round(normalized * 100)}%`}
    >
      {SEGMENT_HEIGHTS.map((height, i) => (
        <div
          key={i}
          className="rounded-[1px]"
          style={{
            width: '3px',
            height: `${height}px`,
            backgroundColor: getSegmentColor(i, filledCount),
          }}
        />
      ))}
    </div>
  );
}
