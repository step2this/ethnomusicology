'use client';

// Map of Camelot key codes to their CSS variable names
const CAMELOT_KEYS = [
  '1A', '1B', '2A', '2B', '3A', '3B', '4A', '4B',
  '5A', '5B', '6A', '6B', '7A', '7B', '8A', '8B',
  '9A', '9B', '10A', '10B', '11A', '11B', '12A', '12B',
];

function getContrastColor(hex: string): string {
  // Parse hex and compute relative luminance
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const luminance = 0.299 * r + 0.587 * g + 0.114 * b;
  return luminance > 0.5 ? '#0F0D0B' : '#F0ECE4';
}

// Fallback colors for when CSS variables aren't available (e.g., in tests)
const FALLBACK_COLORS: Record<string, string> = {
  '1A': '#C04A6E', '1B': '#D06080', '2A': '#C07A4A', '2B': '#D09060',
  '3A': '#C0A84A', '3B': '#D0B860', '4A': '#9BC04A', '4B': '#C04A6E',
  '5A': '#C0A84A', '5B': '#80D060', '6A': '#4A9B6E', '6B': '#60B080',
  '7A': '#6E9B4A', '7B': '#60D0B8', '8A': '#4A8BC0', '8B': '#4A8BC0',
  '9A': '#4AC0A8', '9B': '#6078D0', '10A': '#6E4AC0', '10B': '#8060D0',
  '11A': '#9B4AC0', '11B': '#9B6EC0', '12A': '#C04A9B', '12B': '#D060B0',
};

export function CamelotChip({ camelotCode }: { camelotCode: string | null | undefined }) {
  if (!camelotCode) {
    return <span className="text-xs text-muted-foreground">-</span>;
  }

  const normalized = camelotCode.toUpperCase();
  const isKnown = CAMELOT_KEYS.includes(normalized);

  if (!isKnown) {
    return (
      <span className="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
        {camelotCode}
      </span>
    );
  }

  const fallbackColor = FALLBACK_COLORS[normalized] ?? '#8B8580';
  const textColor = getContrastColor(fallbackColor);

  return (
    <span
      className="inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium leading-none"
      style={{
        backgroundColor: `var(--camelot-${normalized}, ${fallbackColor})`,
        color: textColor,
      }}
      title={`Camelot key: ${normalized}`}
    >
      {normalized}
    </span>
  );
}
