'use client';

import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { Suspense } from 'react';

function HomeContent() {
  const searchParams = useSearchParams();
  const spotifyConnected = searchParams.get('spotify') === 'connected';

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-8">
      {spotifyConnected && (
        <div className="bg-status-success/20 border border-status-success/30 rounded-lg p-4 text-status-success">
          Spotify connected successfully!
        </div>
      )}

      <div className="text-center space-y-4 py-12">
        <h1 className="text-4xl font-bold text-primary">Tarab Studio</h1>
        <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
          LLM-powered DJ setlist generation. Describe the vibe, get a
          harmonically mixed setlist, preview tracks, and buy what you need.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 max-w-4xl mx-auto">
        <NavCard
          href="/setlist/generate"
          title="Generate Setlist"
          description="Describe the vibe and get an AI-generated setlist with harmonic mixing"
        />
        <NavCard
          href="/setlists"
          title="Setlist Library"
          description="Browse, refine, and manage your saved setlists"
        />
        <NavCard
          href="/crates"
          title="Crates"
          description="Organize setlists into crates — your working DJ library"
        />
        <NavCard
          href="/tracks"
          title="Track Catalog"
          description="Browse your imported track collection"
        />
        <NavCard
          href="/import/spotify"
          title="Import from Spotify"
          description="Connect Spotify and import playlists to seed your catalog"
        />
      </div>
    </div>
  );
}

function NavCard({
  href,
  title,
  description,
}: {
  href: string;
  title: string;
  description: string;
}) {
  return (
    <Link
      href={href}
      className="block p-6 rounded-lg border border-border bg-card hover:bg-card/80 hover:border-primary/30 transition-colors"
    >
      <h2 className="text-lg font-semibold text-foreground mb-2">{title}</h2>
      <p className="text-sm text-muted-foreground">{description}</p>
    </Link>
  );
}

export default function HomePage() {
  return (
    <Suspense>
      <HomeContent />
    </Suspense>
  );
}
