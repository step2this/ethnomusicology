import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getSetlistHistory, refineSetlist, revertSetlist } from '@/lib/api-client';
import type { HistoryResponse } from '@/types';
import { setlistKeys } from './use-setlist';

export const refinementKeys = {
  history: (setlistId: string) => ['refinement', 'history', setlistId] as const,
};

export function useSetlistHistory(setlistId: string) {
  return useQuery<HistoryResponse>({
    queryKey: refinementKeys.history(setlistId),
    queryFn: () => getSetlistHistory(setlistId),
    enabled: !!setlistId,
  });
}

export function useRefineSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ setlistId, message }: { setlistId: string; message: string }) =>
      refineSetlist(setlistId, message),
    onSuccess: (_data, { setlistId }) => {
      queryClient.invalidateQueries({ queryKey: setlistKeys.detail(setlistId) });
      queryClient.invalidateQueries({ queryKey: refinementKeys.history(setlistId) });
    },
  });
}

export function useRevertSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      setlistId,
      versionNumber,
    }: {
      setlistId: string;
      versionNumber: number;
    }) => revertSetlist(setlistId, versionNumber),
    onSuccess: (_data, { setlistId }) => {
      queryClient.invalidateQueries({ queryKey: setlistKeys.detail(setlistId) });
      queryClient.invalidateQueries({ queryKey: refinementKeys.history(setlistId) });
    },
  });
}
