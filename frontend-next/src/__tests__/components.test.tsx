import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MetadataChip } from '@/components/metadata-chip';
import { ConfidenceBadge } from '@/components/confidence-badge';

describe('MetadataChip', () => {
  it('renders label and value', () => {
    render(<MetadataChip label="BPM" value={126} />);
    expect(screen.getByText('BPM:')).toBeInTheDocument();
    expect(screen.getByText('126')).toBeInTheDocument();
  });

  it('returns null for null value', () => {
    const { container } = render(<MetadataChip label="BPM" value={null} />);
    expect(container.firstChild).toBeNull();
  });
});

describe('ConfidenceBadge', () => {
  it('renders high confidence in green', () => {
    render(<ConfidenceBadge confidence="high" />);
    const badge = screen.getByText('High');
    expect(badge).toBeInTheDocument();
    expect(badge.className).toContain('green');
  });

  it('renders medium confidence in amber', () => {
    render(<ConfidenceBadge confidence="medium" />);
    const badge = screen.getByText('Medium');
    expect(badge).toBeInTheDocument();
  });

  it('renders low confidence in red', () => {
    render(<ConfidenceBadge confidence="low" />);
    const badge = screen.getByText('Low');
    expect(badge).toBeInTheDocument();
  });

  it('returns null for null confidence', () => {
    const { container } = render(<ConfidenceBadge confidence={null} />);
    expect(container.firstChild).toBeNull();
  });
});
