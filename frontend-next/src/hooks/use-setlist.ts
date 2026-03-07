import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getSetlist,
  listSetlists,
  generateSetlist,
  arrangeSetlist,
  deleteSetlist,
  updateSetlist,
  duplicateSetlist,
} from '@/lib/api-client';
import type { Setlist, SetlistSummary } from '@/types';

export const setlistKeys = {
  all: ['setlists'] as const,
  lists: () => [...setlistKeys.all, 'list'] as const,
  detail: (id: string) => [...setlistKeys.all, 'detail', id] as const,
};

export function useSetlist(id: string) {
  return useQuery<Setlist>({
    queryKey: setlistKeys.detail(id),
    queryFn: () => getSetlist(id),
    enabled: !!id,
  });
}

export function useSetlists() {
  return useQuery<SetlistSummary[]>({
    queryKey: setlistKeys.lists(),
    queryFn: listSetlists,
  });
}

export function useGenerateSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: generateSetlist,
    onSuccess: (data) => {
      queryClient.setQueryData(setlistKeys.detail(data.id), data);
      queryClient.invalidateQueries({ queryKey: setlistKeys.lists() });
    },
  });
}

export function useArrangeSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, energyProfile }: { id: string; energyProfile?: string }) =>
      arrangeSetlist(id, energyProfile),
    onSuccess: (data) => {
      queryClient.setQueryData(setlistKeys.detail(data.id), data);
    },
  });
}

export function useDeleteSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteSetlist,
    onSuccess: (_data, id) => {
      queryClient.removeQueries({ queryKey: setlistKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: setlistKeys.lists() });
    },
  });
}

export function useUpdateSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      updateSetlist(id, name),
    onSuccess: (data) => {
      queryClient.setQueryData(setlistKeys.detail(data.id), data);
      queryClient.invalidateQueries({ queryKey: setlistKeys.lists() });
    },
  });
}

export function useDuplicateSetlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: duplicateSetlist,
    onSuccess: (data) => {
      queryClient.setQueryData(setlistKeys.detail(data.id), data);
      queryClient.invalidateQueries({ queryKey: setlistKeys.lists() });
    },
  });
}
