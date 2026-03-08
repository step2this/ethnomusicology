import { test, expect } from '@playwright/test';

const mockGeneratedSetlist = {
  id: 'gen-1',
  name: null,
  prompt: 'Deep progressive house for a late-night rooftop set',
  tracks: [
    {
      position: 1,
      title: 'Strings of Life',
      artist: 'Derrick May',
      bpm: 126,
      key: 'Am',
      camelot_code: '8A',
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
      camelot_code: '5A',
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
  score_breakdown: { harmonic_flow: 0.85, energy_arc: 0.7, bpm_consistency: 0.9, total: 0.82 },
  energy_profile: 'journey',
  catalog_percentage: 50,
  catalog_warning: null,
  bpm_warnings: [],
  track_count: 2,
  version_number: 1,
  created_at: '2026-03-07T12:00:00Z',
};

test.describe('Setlist generation flow', () => {
  test('generation form renders with all inputs', async ({ page }) => {
    await page.goto('/setlist/generate');

    await expect(page.getByRole('heading', { name: 'Generate Setlist' })).toBeVisible();
    await expect(page.getByPlaceholder(/Describe the vibe|Deep progressive house/i)).toBeVisible();
    await expect(page.getByText('Energy Profile')).toBeVisible();
    await expect(page.getByText('Warm-Up')).toBeVisible();
    await expect(page.getByText('Peak-Time')).toBeVisible();
    await expect(page.getByText('Journey')).toBeVisible();
    await expect(page.getByText('Steady')).toBeVisible();
    await expect(page.getByText('Set Length')).toBeVisible();
    await expect(page.getByText('Creative mode')).toBeVisible();
    await expect(page.getByText('Verify tracks')).toBeVisible();
    await expect(page.getByPlaceholder('e.g. 120')).toBeVisible();
    await expect(page.getByPlaceholder('e.g. 135')).toBeVisible();
  });

  test('submit button disabled when prompt is empty', async ({ page }) => {
    await page.goto('/setlist/generate');
    const submitButton = page.getByRole('button', { name: /Generate Setlist/i });
    await expect(submitButton).toBeDisabled();
  });

  test('submit button enabled after typing prompt', async ({ page }) => {
    await page.goto('/setlist/generate');
    await page.getByPlaceholder(/Deep progressive house/i).fill('Techno set');
    const submitButton = page.getByRole('button', { name: /Generate Setlist/i });
    await expect(submitButton).toBeEnabled();
  });

  test('form submission generates setlist and shows tracks', async ({ page }) => {
    // Mock the generate API
    await page.route('**/api/setlists/generate', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockGeneratedSetlist),
      });
    });

    // Mock audio preview search (called for each track)
    await page.route('**/api/audio/search*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          source: null,
          preview_url: null,
          external_url: null,
          search_queries: [],
          uploader_name: null,
          spotify_uri: null,
        }),
      });
    });

    await page.goto('/setlist/generate');
    await page.getByPlaceholder(/Deep progressive house/i).fill('Deep progressive house for a late-night rooftop set');
    await page.getByRole('button', { name: /Generate Setlist/i }).click();

    // Should show the generated setlist with track tiles
    await expect(page.getByText('Strings of Life')).toBeVisible();
    await expect(page.getByText('Derrick May')).toBeVisible();
    await expect(page.getByText('Starry Night')).toBeVisible();
    await expect(page.getByText('Peggy Gou')).toBeVisible();
  });

  test('generated result shows score breakdown', async ({ page }) => {
    await page.route('**/api/setlists/generate', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockGeneratedSetlist),
      });
    });
    await page.route('**/api/audio/search*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ source: null, preview_url: null, external_url: null, search_queries: [], uploader_name: null, spotify_uri: null }),
      });
    });

    await page.goto('/setlist/generate');
    await page.getByPlaceholder(/Deep progressive house/i).fill('Test prompt');
    await page.getByRole('button', { name: /Generate Setlist/i }).click();

    await expect(page.getByText('Score: 0.82')).toBeVisible();
  });

  test('generated result shows save bar', async ({ page }) => {
    await page.route('**/api/setlists/generate', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockGeneratedSetlist),
      });
    });
    await page.route('**/api/audio/search*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ source: null, preview_url: null, external_url: null, search_queries: [], uploader_name: null, spotify_uri: null }),
      });
    });

    await page.goto('/setlist/generate');
    await page.getByPlaceholder(/Deep progressive house/i).fill('Test prompt');
    await page.getByRole('button', { name: /Generate Setlist/i }).click();

    await expect(page.getByPlaceholder('Name your setlist...')).toBeVisible();
    await expect(page.getByRole('button', { name: /Save/i })).toBeVisible();
  });
});
