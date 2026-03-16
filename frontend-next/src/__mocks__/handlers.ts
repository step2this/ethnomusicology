import { http, HttpResponse } from 'msw';
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
} from '@/types';

const BASE = '/api';

// ---- Fixture data ----

export const mockSetlist: Setlist = {
  id: 'set-1',
  name: 'Test Setlist',
  prompt: 'Deep house vibes',
  tracks: [
    {
      position: 1,
      title: 'Strings of Life',
      artist: 'Derrick May',
      bpm: 126,
      key: 'Am',
      camelot: '8A',
      energy: 0.75,
      transition_score: null,
      energy_score: null,
      is_catalog: false,
      catalog_track_id: null,
      spotify_id: null,
      spotify_uri: null,
      bpm_warning: null,
      confidence: 'high',
      verification_notes: null,
    },
    {
      position: 2,
      title: 'Starry Night',
      artist: 'Peggy Gou',
      bpm: 122,
      key: 'Cm',
      camelot: '5A',
      energy: 0.65,
      transition_score: 0.8,
      energy_score: 0.7,
      is_catalog: true,
      catalog_track_id: 'track-1',
      spotify_id: 'sp-123',
      spotify_uri: 'spotify:track:abc',
      bpm_warning: null,
      confidence: 'high',
      verification_notes: null,
    },
  ],
  harmonic_flow_score: 0.85,
  score_breakdown: {
    harmonic_flow: 0.85,
    energy_arc: 0.7,
    bpm_consistency: 0.9,
    total: 0.82,
  },
  energy_profile: 'journey',
  catalog_percentage: 50,
  catalog_warning: null,
  bpm_warnings: [],
  track_count: 2,
  version_number: 1,
  created_at: '2026-03-07T12:00:00Z',
};

export const mockSetlistSummaries: SetlistSummary[] = [
  {
    id: 'set-1',
    name: 'Test Setlist',
    prompt: 'Deep house vibes',
    track_count: 12,
    created_at: '2026-03-07T12:00:00Z',
  },
  {
    id: 'set-2',
    name: null,
    prompt: 'Peak time techno',
    track_count: 15,
    created_at: '2026-03-06T10:00:00Z',
  },
];

export const mockTracks: TrackListResponse = {
  tracks: [
    {
      id: 'track-1',
      title: 'Starry Night',
      artist: 'Peggy Gou',
      bpm: 122,
      key: 'Cm',
      energy: 0.65,
      duration_ms: 320000,
      preview_url: null,
      album_art_url: null,
      spotify_id: 'sp-123',
      source_playlist_id: 'pl-1',
      date_added: '2026-03-01T00:00:00Z',
    },
  ],
  page: 1,
  total_pages: 1,
  total: 1,
};

export const mockCrates: Crate[] = [
  {
    id: 'crate-1',
    name: 'Weekend Set',
    track_count: 25,
    created_at: '2026-03-05T00:00:00Z',
    updated_at: '2026-03-07T00:00:00Z',
  },
];

export const mockCrateDetail: CrateDetail = {
  id: 'crate-1',
  name: 'Weekend Set',
  tracks: [
    {
      id: 'ct-1',
      title: 'Strings of Life',
      artist: 'Derrick May',
      bpm: 126,
      key: 'Am',
      energy: 0.75,
      position: 1,
      setlist_id: 'set-1',
      setlist_name: 'Test Setlist',
    },
  ],
  created_at: '2026-03-05T00:00:00Z',
  updated_at: '2026-03-07T00:00:00Z',
};

export const mockPurchaseLinks: PurchaseLink[] = [
  { store: 'beatport', name: 'Beatport', url: 'https://www.beatport.com/search?q=test', icon: 'beatport' },
  { store: 'bandcamp', name: 'Bandcamp', url: 'https://bandcamp.com/search?q=test', icon: 'bandcamp' },
  { store: 'juno', name: 'Juno Download', url: 'https://www.junodownload.com/search/?q=test', icon: 'juno' },
  { store: 'traxsource', name: 'Traxsource', url: 'https://www.traxsource.com/search?term=test', icon: 'traxsource' },
];

export const mockPreviewResult: PreviewSearchResult = {
  source: 'deezer',
  preview_url: 'https://cdns-preview.dzcdn.net/test.mp3',
  external_url: 'https://deezer.com/track/123',
  search_queries: ['artist:"Derrick May" track:"Strings of Life"'],
  uploader_name: null,
  spotify_uri: null,
};

export const mockHistory: HistoryResponse = {
  versions: [
    { version_number: 1, action_type: 'generate', summary: 'Initial generation', track_count: 12, created_at: '2026-03-07T12:00:00Z' },
  ],
  conversation: [
    { role: 'user', content: 'Deep house vibes', version_number: null },
    { role: 'assistant', content: 'Generated 12 tracks', version_number: 1 },
  ],
};

// ---- Handlers ----

export const handlers = [
  // Setlists
  http.post(`${BASE}/setlists/generate`, () =>
    HttpResponse.json(mockSetlist),
  ),
  http.get(`${BASE}/setlists`, () =>
    HttpResponse.json({ setlists: mockSetlistSummaries }),
  ),
  http.get(`${BASE}/setlists/:id`, () =>
    HttpResponse.json(mockSetlist),
  ),
  http.delete(`${BASE}/setlists/:id`, () =>
    new HttpResponse(null, { status: 204 }),
  ),
  http.patch(`${BASE}/setlists/:id`, () =>
    HttpResponse.json(mockSetlist),
  ),
  http.post(`${BASE}/setlists/:id/duplicate`, () =>
    HttpResponse.json({ ...mockSetlist, id: 'set-dup' }),
  ),
  http.post(`${BASE}/setlists/:id/arrange`, () =>
    HttpResponse.json(mockSetlist),
  ),

  // Refinement
  http.post(`${BASE}/setlists/:id/refine`, () =>
    HttpResponse.json({
      version_number: 2,
      tracks: mockSetlist.tracks,
      explanation: 'Added more energy',
      change_warning: null,
    } satisfies RefinementResponse),
  ),
  http.post(`${BASE}/setlists/:id/revert/:version`, () =>
    HttpResponse.json({
      version_number: 1,
      tracks: mockSetlist.tracks,
      explanation: 'Reverted to v1',
      change_warning: null,
    } satisfies RefinementResponse),
  ),
  http.get(`${BASE}/setlists/:id/history`, () =>
    HttpResponse.json(mockHistory),
  ),

  // Tracks
  http.get(`${BASE}/tracks`, () =>
    HttpResponse.json(mockTracks),
  ),

  // Crates
  http.get(`${BASE}/crates`, () =>
    HttpResponse.json({ crates: mockCrates }),
  ),
  http.post(`${BASE}/crates`, () =>
    HttpResponse.json(mockCrates[0]),
  ),
  http.get(`${BASE}/crates/:id`, () =>
    HttpResponse.json(mockCrateDetail),
  ),
  http.delete(`${BASE}/crates/:id`, () =>
    new HttpResponse(null, { status: 204 }),
  ),
  http.post(`${BASE}/crates/:crateId/setlists/:setlistId`, () =>
    new HttpResponse(null, { status: 204 }),
  ),
  http.delete(`${BASE}/crates/:crateId/tracks/:trackId`, () =>
    new HttpResponse(null, { status: 204 }),
  ),

  // Purchase links
  http.get(`${BASE}/purchase-links`, () =>
    HttpResponse.json({ links: mockPurchaseLinks }),
  ),

  // Audio
  http.get(`${BASE}/audio/search`, () =>
    HttpResponse.json(mockPreviewResult),
  ),

  // Spotify auth
  http.get(`${BASE}/auth/spotify/status`, () =>
    HttpResponse.json({ connected: false }),
  ),
  http.get(`${BASE}/auth/spotify`, () =>
    HttpResponse.json({ redirect_url: 'https://accounts.spotify.com/authorize?...' }),
  ),

  // Import
  http.post(`${BASE}/import/spotify`, () =>
    HttpResponse.json({ import_id: 'imp-1', status: 'complete' }),
  ),
  http.get(`${BASE}/import/:id`, () =>
    HttpResponse.json({ status: 'complete', total: 50, inserted: 45, updated: 5, failed: 0 }),
  ),
];
