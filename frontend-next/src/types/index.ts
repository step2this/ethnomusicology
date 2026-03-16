// Track from catalog
export interface Track {
  id: string;
  title: string;
  artist: string;
  bpm: number | null;
  key: string | null;
  energy: number | null;
  duration_ms: number | null;
  preview_url: string | null;
  album_art_url: string | null;
  spotify_id: string | null;
  source_playlist_id: string | null;
  date_added: string | null;
}

// Track within a setlist
export interface SetlistTrack {
  position: number;
  title: string;
  artist: string;
  bpm: number | null;
  key: string | null;
  camelot: string | null;
  energy: number | null;
  transition_score: number | null;
  transition_note: string | null;
  energy_score: number | null;
  is_catalog: boolean;
  catalog_track_id: string | null;
  source: string | null;
  track_id: string | null;
  spotify_id: string | null;
  spotify_uri: string | null;
  bpm_warning: string | null;
  confidence: string | null;
  verification_notes: string | null;
  verification_note: string | null;
}

export interface ScoreBreakdown {
  harmonic_flow: number | null;
  energy_arc: number | null;
  bpm_consistency: number | null;
  total: number | null;
}

export interface Setlist {
  id: string;
  name: string | null;
  prompt: string;
  tracks: SetlistTrack[];
  harmonic_flow_score: number | null;
  score_breakdown: ScoreBreakdown | null;
  energy_profile: string | null;
  catalog_percentage: number | null;
  catalog_warning: string | null;
  bpm_warnings: string[];
  track_count: number;
  version_number: number | null;
  created_at: string | null;
}

export interface SetlistSummary {
  id: string;
  name: string | null;
  prompt: string;
  track_count: number;
  created_at: string;
}

export interface TrackListResponse {
  data: Track[];
  page: number;
  per_page: number;
  total_pages: number;
  total: number;
}

// Crates
export interface Crate {
  id: string;
  name: string;
  track_count: number;
  created_at: string;
  updated_at: string;
}

export interface CrateTrack {
  id: string;
  title: string;
  artist: string;
  bpm: number | null;
  key: string | null;
  energy: number | null;
  position: number;
  setlist_id: string;
  setlist_name: string | null;
}

export interface CrateDetail {
  id: string;
  name: string;
  tracks: CrateTrack[];
  created_at: string;
  updated_at: string;
}

// Purchase links
export interface PurchaseLink {
  store: string;
  name: string;
  url: string;
  icon: string;
}

// Refinement
export interface RefinementResponse {
  version_number: number;
  tracks: SetlistTrack[];
  explanation: string;
  change_warning: string | null;
}

export interface SetlistVersion {
  version_number: number;
  action_type: string;
  summary: string | null;
  track_count: number;
  created_at: string;
}

export interface ConversationMessage {
  role: 'user' | 'assistant';
  content: string;
  version_number: number | null;
}

export interface HistoryResponse {
  versions: SetlistVersion[];
  conversation: ConversationMessage[];
}

// Audio preview
export interface PreviewSearchResult {
  source: string | null;
  preview_url: string | null;
  external_url: string | null;
  search_queries: string[];
  uploader_name: string | null;
  spotify_uri: string | null;
}

// API error
export interface ImportStatusResponse {
  import_id: string;
  status: string;
  total_tracks: number | null;
  processed_tracks: number | null;
  error: string | null;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}
