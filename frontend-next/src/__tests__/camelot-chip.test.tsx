import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CamelotChip } from '@/components/camelot-chip';

describe('CamelotChip', () => {
  it('renders known Camelot key with color', () => {
    render(<CamelotChip camelotCode="8A" />);
    const chip = screen.getByText('8A');
    expect(chip).toBeInTheDocument();
    expect(chip).toHaveAttribute('title', 'Camelot key: 8A');
  });

  it('normalizes lowercase input', () => {
    render(<CamelotChip camelotCode="11b" />);
    expect(screen.getByText('11B')).toBeInTheDocument();
  });

  it('renders unknown key with muted style (normalized to uppercase)', () => {
    render(<CamelotChip camelotCode="z9" />);
    const chip = screen.getByText('Z9');
    expect(chip).toBeInTheDocument();
    expect(chip.className).toContain('text-muted-foreground');
  });

  it('truncates unknown key longer than 4 chars', () => {
    render(<CamelotChip camelotCode="ABCDEF" />);
    expect(screen.getByText('ABCD')).toBeInTheDocument();
  });

  it('renders dash for null input', () => {
    render(<CamelotChip camelotCode={null} />);
    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('renders dash for undefined input', () => {
    render(<CamelotChip camelotCode={undefined} />);
    expect(screen.getByText('-')).toBeInTheDocument();
  });
});
