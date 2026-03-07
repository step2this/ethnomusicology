import { useQuery } from '@tanstack/react-query';
import { listTracks, searchPreview } from '@/lib/api-client';
import type { TrackListResponse, PreviewSearchResult } from '@/types';

export const trackKeys = {
  all: ['tracks'] as const,
  list: (options?: { page?: number; perPage?: number; sort?: string; order?: string }) =>
    [...trackKeys.all, 'list', options] as const,
};

export function useTracks(options?: {
  page?: number;
  perPage?: number;
  sort?: string;
  order?: string;
}) {
  return useQuery<TrackListResponse>({
    queryKey: trackKeys.list(options),
    queryFn: () => listTracks(options),
  });
}

export function useSearchPreview(): (
  title: string,
  artist: string,
) => Promise<PreviewSearchResult> {
  return searchPreview;
}
