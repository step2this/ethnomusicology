import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EnergyBar } from '@/components/energy-bar';

describe('EnergyBar', () => {
  it('renders 5 segments', () => {
    render(<EnergyBar energy={0.7} />);
    const bars = screen.getAllByTestId('energy-segment');
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

  it('renders 0 filled segments for energy=0', () => {
    render(<EnergyBar energy={0} />);
    expect(screen.getByTitle('Energy: 0%')).toBeInTheDocument();
    const segments = screen.getAllByTestId('energy-segment');
    expect(segments).toHaveLength(5);
    const filledCount = segments.filter((s) => s.getAttribute('data-filled') === 'true').length;
    expect(filledCount).toBe(0);
  });

  it('renders all segments filled for energy=1 (0-1 scale)', () => {
    render(<EnergyBar energy={1} />);
    expect(screen.getByTitle('Energy: 100%')).toBeInTheDocument();
    const segments = screen.getAllByTestId('energy-segment');
    expect(segments).toHaveLength(5);
    const filledCount = segments.filter((s) => s.getAttribute('data-filled') === 'true').length;
    expect(filledCount).toBe(5);
  });

  it('renders all segments filled for energy=10 (0-10 scale)', () => {
    render(<EnergyBar energy={10} />);
    expect(screen.getByTitle('Energy: 100%')).toBeInTheDocument();
    const segments = screen.getAllByTestId('energy-segment');
    expect(segments).toHaveLength(5);
    const filledCount = segments.filter((s) => s.getAttribute('data-filled') === 'true').length;
    expect(filledCount).toBe(5);
  });

  it('handles negative energy gracefully', () => {
    render(<EnergyBar energy={-1} />);
    const segments = screen.getAllByTestId('energy-segment');
    expect(segments).toHaveLength(5);
    const filledCount = segments.filter((s) => s.getAttribute('data-filled') === 'true').length;
    expect(filledCount).toBe(0);
  });

  it('handles energy > 10 gracefully', () => {
    render(<EnergyBar energy={15} />);
    const segments = screen.getAllByTestId('energy-segment');
    expect(segments).toHaveLength(5);
    // Should still render all 5 segments without crashing
  });
});
