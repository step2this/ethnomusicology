import { useQuery, useMutation } from '@tanstack/react-query';
import {
  checkSpotifyConnection,
  getSpotifyAuthUrl,
  importSpotifyPlaylist,
} from '@/lib/api-client';

export const spotifyKeys = {
  connection: (userId: string) => ['spotify', 'connection', userId] as const,
};

export function useSpotifyConnection(userId: string) {
  return useQuery<boolean>({
    queryKey: spotifyKeys.connection(userId),
    queryFn: () => checkSpotifyConnection(userId),
    enabled: !!userId,
  });
}

export function useSpotifyAuthUrl() {
  return useMutation({
    mutationFn: (userId: string) => getSpotifyAuthUrl(userId),
  });
}

export function useImportPlaylist() {
  return useMutation({
    mutationFn: (playlistUrl: string) => importSpotifyPlaylist(playlistUrl),
  });
}
