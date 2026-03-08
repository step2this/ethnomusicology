import { useCallback, useEffect } from 'react';
import { searchPreview } from '@/lib/api-client';
import { usePlaybackStore } from '@/stores/playback-store';
import type { SetlistTrack } from '@/types';

export function usePrefetchPreviews(tracks: SetlistTrack[] | undefined) {
  const { setPreviewUrl, setTrackCount } = usePlaybackStore();

  const prefetch = useCallback(
    (items: SetlistTrack[]) => {
      setTrackCount(items.length);
      items.forEach((track, i) => {
        searchPreview(track.title, track.artist).then((result) => {
          if (result.preview_url) {
            setPreviewUrl(i, result.preview_url);
          }
        });
      });
    },
    [setPreviewUrl, setTrackCount],
  );

  useEffect(() => {
    if (tracks && tracks.length > 0) {
      prefetch(tracks);
    }
  }, [tracks, prefetch]);
}
