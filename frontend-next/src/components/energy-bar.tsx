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

/**
 * Visualizes track energy as a 5-segment bar chart.
 *
 * @param energy - Accepts two input ranges:
 *   - **0-1 normalized** (e.g., 0.7 = 70% energy)
 *   - **0-10 integer scale** (e.g., 7 = 70% energy) — values > 1 are divided by 10
 *
 * **Edge case:** `energy=1` is ambiguous — it is treated as 1.0 on the 0-1 scale
 * (i.e., 100% energy), NOT as 1 on the 0-10 scale (which would be 10%).
 * If the backend ever sends 1 meaning "low energy on a 0-10 scale", this will
 * be misinterpreted as maximum energy.
 */
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
          data-testid="energy-segment"
          data-filled={i < filledCount}
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
