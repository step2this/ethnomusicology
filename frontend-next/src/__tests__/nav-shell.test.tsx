import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

let mockPathname = '/';

vi.mock('next/navigation', () => ({
  usePathname: () => mockPathname,
}));

// Mock next/link to render a plain anchor
vi.mock('next/link', () => ({
  default: ({ href, children, ...props }: { href: string; children: React.ReactNode; [key: string]: unknown }) => (
    <a href={href} {...props}>{children}</a>
  ),
}));

import { NavShell } from '@/components/nav-shell';

describe('NavShell', () => {
  it('renders all nav links', () => {
    mockPathname = '/';
    render(<NavShell><div>content</div></NavShell>);
    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Generate')).toBeInTheDocument();
    expect(screen.getByText('Library')).toBeInTheDocument();
    expect(screen.getByText('Crates')).toBeInTheDocument();
    expect(screen.getByText('Catalog')).toBeInTheDocument();
    expect(screen.getByText('Import')).toBeInTheDocument();
  });

  it('renders brand name', () => {
    mockPathname = '/';
    render(<NavShell><div>content</div></NavShell>);
    expect(screen.getByText('Tarab Studio')).toBeInTheDocument();
  });

  it('renders children', () => {
    mockPathname = '/';
    render(<NavShell><div>test content</div></NavShell>);
    expect(screen.getByText('test content')).toBeInTheDocument();
  });

  it('highlights Home link when on root path', () => {
    mockPathname = '/';
    render(<NavShell><div>content</div></NavShell>);
    const homeLink = screen.getByText('Home');
    expect(homeLink.className).toContain('text-primary');
    expect(homeLink.className).toContain('font-medium');
  });

  it('highlights Generate link when on /setlist/generate', () => {
    mockPathname = '/setlist/generate';
    render(<NavShell><div>content</div></NavShell>);
    const genLink = screen.getByText('Generate');
    expect(genLink.className).toContain('text-primary');
  });

  it('highlights Library link when on /setlists', () => {
    mockPathname = '/setlists';
    render(<NavShell><div>content</div></NavShell>);
    const libLink = screen.getByText('Library');
    expect(libLink.className).toContain('text-primary');
  });

  it('highlights Library link on nested setlist path', () => {
    mockPathname = '/setlists/set-1';
    render(<NavShell><div>content</div></NavShell>);
    const libLink = screen.getByText('Library');
    expect(libLink.className).toContain('text-primary');
  });

  it('does not highlight Home when on other pages', () => {
    mockPathname = '/crates';
    render(<NavShell><div>content</div></NavShell>);
    const homeLink = screen.getByText('Home');
    expect(homeLink.className).toContain('text-muted-foreground');
  });

  it('nav links have correct hrefs', () => {
    mockPathname = '/';
    render(<NavShell><div>content</div></NavShell>);
    expect(screen.getByText('Home').closest('a')).toHaveAttribute('href', '/');
    expect(screen.getByText('Generate').closest('a')).toHaveAttribute('href', '/setlist/generate');
    expect(screen.getByText('Library').closest('a')).toHaveAttribute('href', '/setlists');
    expect(screen.getByText('Crates').closest('a')).toHaveAttribute('href', '/crates');
    expect(screen.getByText('Catalog').closest('a')).toHaveAttribute('href', '/tracks');
    expect(screen.getByText('Import').closest('a')).toHaveAttribute('href', '/import/spotify');
  });
});
