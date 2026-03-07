'use client';

import { useState } from 'react';
import Link from 'next/link';
import { CheckCircle, CloudOff, Download, ExternalLink, Loader2, AlertCircle, Music } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useSpotifyConnection, useSpotifyAuthUrl, useImportPlaylist } from '@/hooks/use-spotify';

export default function SpotifyImportPage() {
  const [playlistUrl, setPlaylistUrl] = useState('');
  const connection = useSpotifyConnection('default-user');
  const authUrl = useSpotifyAuthUrl();
  const importPlaylist = useImportPlaylist();

  const isConnected = connection.data === true;
  const isCheckingConnection = connection.isLoading;

  function handleConnect() {
    authUrl.mutate('default-user', {
      onSuccess: (url) => {
        window.location.href = url;
      },
    });
  }

  function handleImport() {
    const trimmed = playlistUrl.trim();
    if (!trimmed) return;
    importPlaylist.mutate(trimmed);
  }

  return (
    <div className="mx-auto max-w-2xl px-6 py-10">
      <h1 className="mb-8 text-2xl font-bold text-foreground">Import from Spotify</h1>

      {/* Connection Status Card */}
      <div className="mb-6 rounded-lg border border-border bg-card p-4">
        <div className="flex items-center gap-4">
          {isCheckingConnection ? (
            <Loader2 className="size-8 animate-spin text-muted-foreground" />
          ) : isConnected ? (
            <CheckCircle className="size-8 text-primary" />
          ) : (
            <CloudOff className="size-8 text-muted-foreground" />
          )}
          <div className="flex-1">
            <p className="text-sm font-medium text-foreground">Spotify Connection</p>
            <p className="text-sm text-muted-foreground">
              {isCheckingConnection
                ? 'Checking connection...'
                : isConnected
                  ? 'Connected and ready to import.'
                  : 'Not connected. Click Connect to authorize.'}
            </p>
          </div>
          {!isCheckingConnection && !isConnected && (
            <Button
              onClick={handleConnect}
              disabled={authUrl.isPending}
              variant="secondary"
            >
              {authUrl.isPending ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                'Connect'
              )}
            </Button>
          )}
        </div>
        {connection.isError && (
          <p className="mt-2 text-sm text-destructive">
            Failed to check connection status.
          </p>
        )}
      </div>

      {/* Import Form */}
      {isConnected && (
        <div className="mb-6 space-y-4">
          <p className="text-sm font-medium text-foreground">
            Paste a Spotify playlist URL or URI
          </p>
          <div className="flex gap-3">
            <input
              type="text"
              value={playlistUrl}
              onChange={(e) => setPlaylistUrl(e.target.value)}
              placeholder="https://open.spotify.com/playlist/... or spotify:playlist:..."
              disabled={importPlaylist.isPending}
              className="flex-1 rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-2 focus:ring-ring/50 disabled:opacity-50"
            />
            <Button
              onClick={handleImport}
              disabled={importPlaylist.isPending || !playlistUrl.trim()}
            >
              {importPlaylist.isPending ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <Download className="size-4" />
              )}
              {importPlaylist.isPending ? 'Importing...' : 'Import'}
            </Button>
          </div>
        </div>
      )}

      {/* Import Progress / Loading */}
      {importPlaylist.isPending && (
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center gap-3">
            <Loader2 className="size-5 animate-spin text-primary" />
            <p className="text-sm text-foreground">Importing playlist...</p>
          </div>
        </div>
      )}

      {/* Import Success */}
      {importPlaylist.isSuccess && (
        <div className="rounded-lg border border-primary/30 bg-primary/5 p-6">
          <div className="flex items-center gap-3 mb-4">
            <CheckCircle className="size-5 text-primary" />
            <p className="text-sm font-medium text-foreground">Import complete!</p>
          </div>
          <div className="mb-4 grid grid-cols-2 gap-3 text-sm">
            <div className="rounded-md bg-background/50 p-3">
              <p className="text-muted-foreground">Import ID</p>
              <p className="font-mono text-foreground">{importPlaylist.data.import_id}</p>
            </div>
            <div className="rounded-md bg-background/50 p-3">
              <p className="text-muted-foreground">Status</p>
              <p className="font-medium text-foreground">{importPlaylist.data.status}</p>
            </div>
          </div>
          <Link href="/tracks">
            <Button variant="secondary" className="gap-2">
              <Music className="size-4" />
              View your tracks
              <ExternalLink className="size-3" />
            </Button>
          </Link>
        </div>
      )}

      {/* Import Error */}
      {importPlaylist.isError && (
        <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-6">
          <div className="flex items-center gap-3 mb-3">
            <AlertCircle className="size-5 text-destructive" />
            <p className="text-sm font-medium text-destructive">Import Failed</p>
          </div>
          <p className="mb-4 text-sm text-muted-foreground">
            {importPlaylist.error instanceof Error
              ? importPlaylist.error.message
              : 'An unknown error occurred.'}
          </p>
          <Button
            variant="secondary"
            onClick={() => importPlaylist.reset()}
          >
            Try Again
          </Button>
        </div>
      )}
    </div>
  );
}
