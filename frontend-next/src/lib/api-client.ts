import type {
  Setlist,
  SetlistSummary,
  TrackListResponse,
  Crate,
  CrateDetail,
  PurchaseLink,
  PreviewSearchResult,
  RefinementResponse,
  HistoryResponse,
  ImportStatusResponse,
  ApiError,
} from '@/types';

const BASE_URL = '/api';
const DEFAULT_TIMEOUT = 10_000;
const GENERATE_TIMEOUT = 120_000;

class ApiClientError extends Error {
  code: string;
  constructor(code: string, message: string) {
    super(message);
    this.name = 'ApiClientError';
    this.code = code;
  }
}

async function request<T>(
  path: string,
  options: RequestInit & { timeout?: number; params?: Record<string, string | number> } = {},
): Promise<T> {
  const { timeout = DEFAULT_TIMEOUT, params, ...fetchOptions } = options;

  let url = `${BASE_URL}${path}`;
  if (params) {
    const searchParams = new URLSearchParams();
    for (const [key, value] of Object.entries(params)) {
      searchParams.set(key, String(value));
    }
    url += `?${searchParams.toString()}`;
  }

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(url, {
      ...fetchOptions,
      signal: controller.signal,
      headers: {
        ...(fetchOptions.body ? { 'Content-Type': 'application/json' } : {}),
        ...fetchOptions.headers,
      },
    });

    if (!response.ok) {
      let errorData: ApiError | null = null;
      try {
        errorData = await response.json();
      } catch {
        // non-JSON error
      }
      throw new ApiClientError(
        errorData?.error?.code ?? 'UNKNOWN',
        errorData?.error?.message ?? `Request failed: ${response.status}`,
      );
    }

    // 204 No Content
    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  } finally {
    clearTimeout(timer);
  }
}

// ---------------------------------------------------------------------------
// Spotify OAuth
// ---------------------------------------------------------------------------

export async function checkSpotifyConnection(userId: string): Promise<boolean> {
  const data = await request<{ connected: boolean }>('/auth/spotify/status', {
    headers: { 'X-User-Id': userId },
  });
  return data.connected;
}

export async function getSpotifyAuthUrl(userId: string): Promise<string> {
  const data = await request<{ redirect_url: string }>('/auth/spotify', {
    headers: { 'X-User-Id': userId },
  });
  return data.redirect_url;
}

// ---------------------------------------------------------------------------
// Spotify Import
// ---------------------------------------------------------------------------

export async function importSpotifyPlaylist(
  playlistUrl: string,
): Promise<{ import_id: string; status: string }> {
  return request('/import/spotify', {
    method: 'POST',
    body: JSON.stringify({ playlist_url: playlistUrl }),
  });
}

export async function getImportStatus(
  importId: string,
): Promise<ImportStatusResponse> {
  return request(`/import/${importId}`);
}

// ---------------------------------------------------------------------------
// Track Catalog
// ---------------------------------------------------------------------------

export async function listTracks(options?: {
  page?: number;
  perPage?: number;
  sort?: string;
  order?: string;
}): Promise<TrackListResponse> {
  return request('/tracks', {
    params: {
      page: options?.page ?? 1,
      per_page: options?.perPage ?? 25,
      sort: options?.sort ?? 'date_added',
      order: options?.order ?? 'desc',
    },
  });
}

// ---------------------------------------------------------------------------
// Setlist Generation
// ---------------------------------------------------------------------------

export async function generateSetlist(options: {
  prompt: string;
  trackCount?: number;
  energyProfile?: string;
  sourcePlaylistId?: string;
  seedTracklist?: string;
  creativeMode?: boolean;
  bpmMin?: number;
  bpmMax?: number;
  verify?: boolean;
}): Promise<Setlist> {
  const body: Record<string, unknown> = { prompt: options.prompt };
  if (options.trackCount != null) body.track_count = options.trackCount;
  if (options.energyProfile != null) body.energy_profile = options.energyProfile;
  if (options.sourcePlaylistId != null) body.source_playlist_id = options.sourcePlaylistId;
  if (options.seedTracklist != null) body.seed_tracklist = options.seedTracklist;
  if (options.creativeMode != null) body.creative_mode = options.creativeMode;
  if (options.bpmMin != null && options.bpmMax != null) {
    body.bpm_range = { min: options.bpmMin, max: options.bpmMax };
  }
  if (options.verify != null) body.verify = options.verify;

  return request('/setlists/generate', {
    method: 'POST',
    body: JSON.stringify(body),
    timeout: GENERATE_TIMEOUT,
  });
}

export async function arrangeSetlist(
  id: string,
  energyProfile?: string,
): Promise<Setlist> {
  const body: Record<string, unknown> = {};
  if (energyProfile) body.energy_profile = energyProfile;

  return request(`/setlists/${id}/arrange`, {
    method: 'POST',
    body: Object.keys(body).length > 0 ? JSON.stringify(body) : undefined,
  });
}

export async function getSetlist(id: string): Promise<Setlist> {
  return request(`/setlists/${id}`);
}

export async function listSetlists(): Promise<SetlistSummary[]> {
  const data = await request<{ setlists: SetlistSummary[] }>('/setlists');
  return data.setlists ?? [];
}

export async function deleteSetlist(id: string): Promise<void> {
  return request(`/setlists/${id}`, { method: 'DELETE' });
}

export async function updateSetlist(
  id: string,
  name: string,
): Promise<Setlist> {
  return request(`/setlists/${id}`, {
    method: 'PATCH',
    body: JSON.stringify({ name }),
  });
}

export async function duplicateSetlist(id: string): Promise<Setlist> {
  return request(`/setlists/${id}/duplicate`, { method: 'POST' });
}

// ---------------------------------------------------------------------------
// Crates
// ---------------------------------------------------------------------------

export async function listCrates(): Promise<Crate[]> {
  const data = await request<{ crates: Crate[] }>('/crates');
  return data.crates ?? [];
}

export async function createCrate(name: string): Promise<Crate> {
  return request('/crates', {
    method: 'POST',
    body: JSON.stringify({ name }),
  });
}

export async function getCrate(id: string): Promise<CrateDetail> {
  return request(`/crates/${id}`);
}

export async function deleteCrate(id: string): Promise<void> {
  return request(`/crates/${id}`, { method: 'DELETE' });
}

export async function addSetlistToCrate(
  crateId: string,
  setlistId: string,
): Promise<void> {
  return request(`/crates/${crateId}/setlists/${setlistId}`, {
    method: 'POST',
  });
}

export async function removeCrateTrack(
  crateId: string,
  trackId: string,
): Promise<void> {
  return request(`/crates/${crateId}/tracks/${trackId}`, {
    method: 'DELETE',
  });
}

// ---------------------------------------------------------------------------
// Purchase Links
// ---------------------------------------------------------------------------

export async function getPurchaseLinks(
  title: string,
  artist: string,
): Promise<PurchaseLink[]> {
  const data = await request<{ links: PurchaseLink[] }>('/purchase-links', {
    params: { title, artist },
  });
  return data.links;
}

// ---------------------------------------------------------------------------
// Audio Preview
// ---------------------------------------------------------------------------

export async function searchPreview(
  title: string,
  artist: string,
): Promise<PreviewSearchResult> {
  try {
    return await request<PreviewSearchResult>('/audio/search', {
      params: { title, artist },
    });
  } catch {
    return {
      source: null,
      preview_url: null,
      external_url: null,
      search_queries: [],
      uploader_name: null,
      spotify_uri: null,
    };
  }
}

// ---------------------------------------------------------------------------
// Refinement
// ---------------------------------------------------------------------------

export async function refineSetlist(
  setlistId: string,
  message: string,
): Promise<RefinementResponse> {
  return request(`/setlists/${setlistId}/refine`, {
    method: 'POST',
    body: JSON.stringify({ message }),
    timeout: GENERATE_TIMEOUT,
  });
}

export async function revertSetlist(
  setlistId: string,
  versionNumber: number,
): Promise<RefinementResponse> {
  return request(`/setlists/${setlistId}/revert/${versionNumber}`, {
    method: 'POST',
  });
}

export async function getSetlistHistory(
  setlistId: string,
): Promise<HistoryResponse> {
  return request(`/setlists/${setlistId}/history`);
}

export { ApiClientError };
