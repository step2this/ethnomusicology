'use client';

export function MetadataChip({
  label,
  value,
}: {
  label: string;
  value: string | number | null;
}) {
  if (value === null || value === undefined) return null;

  return (
    <span className="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
      <span className="font-medium text-foreground/70">{label}:</span>
      <span className="ml-1">{value}</span>
    </span>
  );
}
