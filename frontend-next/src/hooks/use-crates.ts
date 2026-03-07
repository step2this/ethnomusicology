import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listCrates,
  getCrate,
  createCrate,
  deleteCrate,
  addSetlistToCrate,
  removeCrateTrack,
} from '@/lib/api-client';
import type { Crate, CrateDetail } from '@/types';

export const crateKeys = {
  all: ['crates'] as const,
  lists: () => [...crateKeys.all, 'list'] as const,
  detail: (id: string) => [...crateKeys.all, 'detail', id] as const,
};

export function useCrates() {
  return useQuery<Crate[]>({
    queryKey: crateKeys.lists(),
    queryFn: listCrates,
  });
}

export function useCrate(id: string) {
  return useQuery<CrateDetail>({
    queryKey: crateKeys.detail(id),
    queryFn: () => getCrate(id),
    enabled: !!id,
  });
}

export function useCreateCrate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: createCrate,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: crateKeys.lists() });
    },
  });
}

export function useDeleteCrate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteCrate,
    onSuccess: (_data, id) => {
      queryClient.removeQueries({ queryKey: crateKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: crateKeys.lists() });
    },
  });
}

export function useAddSetlistToCrate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ crateId, setlistId }: { crateId: string; setlistId: string }) =>
      addSetlistToCrate(crateId, setlistId),
    onSuccess: (_data, { crateId }) => {
      queryClient.invalidateQueries({ queryKey: crateKeys.detail(crateId) });
    },
  });
}

export function useRemoveCrateTrack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ crateId, trackId }: { crateId: string; trackId: string }) =>
      removeCrateTrack(crateId, trackId),
    onSuccess: (_data, { crateId }) => {
      queryClient.invalidateQueries({ queryKey: crateKeys.detail(crateId) });
    },
  });
}
