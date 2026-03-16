import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EnergyBar } from '@/components/energy-bar';

describe('EnergyBar', () => {
  it('renders 5 segments', () => {
    const { container } = render(<EnergyBar energy={0.7} />);
    const segments = container.querySelectorAll('[style]');
    // The wrapper div also has style, but segments are the children with width: 3px
    const bars = Array.from(segments).filter(el => (el as HTMLElement).style.width === '3px');
    expect(bars).toHaveLength(5);
  });

  it('displays correct tooltip for percentage energy', () => {
    render(<EnergyBar energy={0.8} />);
    expect(screen.getByTitle('Energy: 80%')).toBeInTheDocument();
  });

  it('handles energy as 0-10 scale', () => {
    render(<EnergyBar energy={7} />);
    expect(screen.getByTitle('Energy: 70%')).toBeInTheDocument();
  });

  it('renders dash for null energy', () => {
    render(<EnergyBar energy={null} />);
    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('renders dash for undefined energy', () => {
    render(<EnergyBar energy={undefined} />);
    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('has accessible label', () => {
    render(<EnergyBar energy={0.5} />);
    expect(screen.getByLabelText('Energy: 50%')).toBeInTheDocument();
  });
});
