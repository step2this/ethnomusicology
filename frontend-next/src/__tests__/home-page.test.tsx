import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

let mockSearchParams = new URLSearchParams();

vi.mock('next/navigation', () => ({
  usePathname: () => '/',
  useSearchParams: () => mockSearchParams,
  useParams: () => ({}),
}));

vi.mock('next/link', () => ({
  default: ({ href, children, ...props }: { href: string; children: React.ReactNode; [key: string]: unknown }) => (
    <a href={href} {...props}>{children}</a>
  ),
}));

import HomePage from '@/app/page';

describe('HomePage', () => {
  beforeEach(() => {
    mockSearchParams = new URLSearchParams();
  });

  it('renders title', () => {
    render(<HomePage />);
    expect(screen.getByText('Tarab Studio')).toBeInTheDocument();
  });

  it('renders description text', () => {
    render(<HomePage />);
    expect(screen.getByText(/LLM-powered DJ setlist generation/)).toBeInTheDocument();
  });

  it('shows Generate Setlist card', () => {
    render(<HomePage />);
    expect(screen.getByText('Generate Setlist')).toBeInTheDocument();
    const card = screen.getByText('Generate Setlist').closest('a');
    expect(card).toHaveAttribute('href', '/setlist/generate');
  });

  it('shows Setlist Library card', () => {
    render(<HomePage />);
    expect(screen.getByText('Setlist Library')).toBeInTheDocument();
  });

  it('shows Crates card', () => {
    render(<HomePage />);
    expect(screen.getByText('Crates')).toBeInTheDocument();
  });

  it('shows Track Catalog card', () => {
    render(<HomePage />);
    expect(screen.getByText('Track Catalog')).toBeInTheDocument();
  });

  it('shows Import from Spotify card', () => {
    render(<HomePage />);
    expect(screen.getByText('Import from Spotify')).toBeInTheDocument();
  });

  it('nav cards have correct links', () => {
    render(<HomePage />);
    expect(screen.getByText('Generate Setlist').closest('a')).toHaveAttribute('href', '/setlist/generate');
    expect(screen.getByText('Setlist Library').closest('a')).toHaveAttribute('href', '/setlists');
    expect(screen.getByText('Crates').closest('a')).toHaveAttribute('href', '/crates');
    expect(screen.getByText('Track Catalog').closest('a')).toHaveAttribute('href', '/tracks');
    expect(screen.getByText('Import from Spotify').closest('a')).toHaveAttribute('href', '/import/spotify');
  });

  it('shows Spotify connected message when ?spotify=connected', () => {
    mockSearchParams = new URLSearchParams('spotify=connected');
    render(<HomePage />);
    expect(screen.getByText('Spotify connected successfully!')).toBeInTheDocument();
  });

  it('does not show Spotify connected message by default', () => {
    render(<HomePage />);
    expect(screen.queryByText('Spotify connected successfully!')).not.toBeInTheDocument();
  });

  it('does not show Spotify message with other param values', () => {
    mockSearchParams = new URLSearchParams('spotify=disconnected');
    render(<HomePage />);
    expect(screen.queryByText('Spotify connected successfully!')).not.toBeInTheDocument();
  });
});
