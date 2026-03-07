'use client';

import { useState } from 'react';
import { ShoppingBag, ChevronDown, ChevronUp, Loader2 } from 'lucide-react';
import { usePurchaseLinks } from '@/hooks/use-purchase-links';

const storeIcons: Record<string, string> = {
  beatport: '\uD83C\uDFA7',
  bandcamp: '\uD83C\uDFB5',
  juno: '\uD83D\uDCBF',
  traxsource: '\uD83C\uDFB6',
};

export function PurchaseLinkPanel({
  title,
  artist,
}: {
  title: string;
  artist: string;
}) {
  const [expanded, setExpanded] = useState(false);
  const { data: links, isLoading } = usePurchaseLinks(title, artist, expanded);

  if (!title && !artist) return null;

  return (
    <div className="mt-1">
      <button
        onClick={() => setExpanded(!expanded)}
        className="inline-flex items-center gap-1 rounded px-1 py-0.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
      >
        <ShoppingBag className="h-3.5 w-3.5" />
        <span>Buy</span>
        {expanded ? (
          <ChevronUp className="h-3.5 w-3.5" />
        ) : (
          <ChevronDown className="h-3.5 w-3.5" />
        )}
      </button>

      {expanded && (
        <div className="mt-1">
          {isLoading && (
            <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
          )}

          {!isLoading && links && links.length === 0 && (
            <span className="text-xs text-muted-foreground">No purchase links</span>
          )}

          {!isLoading && links && links.length > 0 && (
            <div className="flex flex-wrap gap-1.5">
              {links.map((link) => (
                <a
                  key={link.store}
                  href={link.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 rounded-full border border-border bg-muted/50 px-2 py-0.5 text-xs text-muted-foreground hover:text-foreground hover:border-foreground/30 transition-colors"
                >
                  <span>{storeIcons[link.icon] ?? '\uD83D\uDD17'}</span>
                  <span>{link.name}</span>
                </a>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
