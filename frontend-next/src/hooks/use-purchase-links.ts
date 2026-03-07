import { useQuery } from '@tanstack/react-query';
import { getPurchaseLinks } from '@/lib/api-client';
import type { PurchaseLink } from '@/types';

export const purchaseKeys = {
  links: (title: string, artist: string) =>
    ['purchase-links', title, artist] as const,
};

export function usePurchaseLinks(
  title: string,
  artist: string,
  enabled: boolean,
) {
  return useQuery<PurchaseLink[]>({
    queryKey: purchaseKeys.links(title, artist),
    queryFn: () => getPurchaseLinks(title, artist),
    enabled: enabled && !!title && !!artist,
  });
}
