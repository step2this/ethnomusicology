import { test, expect } from '@playwright/test';

const mockSetlist = {
  id: 'set-detail-1',
  name: 'Friday Night Set',
  prompt: 'Deep house vibes for the rooftop',
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

const mockHistory = {
  versions: [
    { version_number: 1, action_type: 'generate', summary: 'Initial generation', track_count: 2, created_at: '2026-03-07T12:00:00Z' },
  ],
  conversation: [
    { role: 'user', content: 'Deep house vibes for the rooftop', version_number: null },
    { role: 'assistant', content: 'Generated 2 tracks', version_number: 1 },
  ],
};

function mockDetailApis(page: import('@playwright/test').Page) {
  return Promise.all([
    page.route('**/api/setlists/set-detail-1', (route) => {
      if (route.request().method() === 'GET') {
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(mockSetlist),
        });
      } else {
        route.continue();
      }
    }),
    page.route('**/api/setlists/set-detail-1/history', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockHistory),
      });
    }),
    page.route('**/api/audio/search*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ source: null, preview_url: null, external_url: null, search_queries: [], uploader_name: null, spotify_uri: null }),
      });
    }),
  ]);
}

test.describe('Setlist detail page', () => {
  test('loads and displays setlist name and tracks', async ({ page }) => {
    await mockDetailApis(page);
    await page.goto('/setlists/set-detail-1');

    await expect(page.getByRole('heading', { name: 'Friday Night Set' })).toBeVisible();
    await expect(page.getByText('Deep house vibes for the rooftop')).toBeVisible();
    await expect(page.getByText('Strings of Life')).toBeVisible();
    await expect(page.getByText('Derrick May')).toBeVisible();
    await expect(page.getByText('Starry Night')).toBeVisible();
    await expect(page.getByText('Peggy Gou')).toBeVisible();
  });

  test('shows metadata: track count, energy profile, version', async ({ page }) => {
    await mockDetailApis(page);
    await page.goto('/setlists/set-detail-1');

    await expect(page.getByText('2 tracks')).toBeVisible();
    await expect(page.getByText('Energy: journey')).toBeVisible();
    await expect(page.getByText('v1')).toBeVisible();
  });

  test('shows score breakdown', async ({ page }) => {
    await mockDetailApis(page);
    await page.goto('/setlists/set-detail-1');

    await expect(page.getByText('Score: 0.82')).toBeVisible();
    await expect(page.getByText('Harmonic: 0.85')).toBeVisible();
    await expect(page.getByText('Energy: 0.7')).toBeVisible();
  });

  test('inline rename: click name shows edit input', async ({ page }) => {
    await mockDetailApis(page);
    await page.goto('/setlists/set-detail-1');

    // Click the setlist name to start editing
    await page.getByRole('heading', { name: 'Friday Night Set' }).click();

    // Should show an input with the current name
    const nameInput = page.locator('input[type="text"]').first();
    await expect(nameInput).toBeVisible();
    await expect(nameInput).toHaveValue('Friday Night Set');
  });

  test('refinement chat panel is visible', async ({ page }) => {
    await mockDetailApis(page);
    await page.goto('/setlists/set-detail-1');

    // The refinement chat should be present (look for the input/placeholder)
    await expect(page.getByPlaceholder(/refine|swap|remove|more energy/i)).toBeVisible();
  });

  test('version history is visible', async ({ page }) => {
    await mockDetailApis(page);
    await page.goto('/setlists/set-detail-1');

    // Version history shows the initial version
    await expect(page.getByText('Initial generation')).toBeVisible();
  });

  test('shows error state for non-existent setlist', async ({ page }) => {
    await page.route('**/api/setlists/nonexistent*', (route) => {
      route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ error: { code: 'NOT_FOUND', message: 'Setlist not found' } }),
      });
    });
    await page.route('**/api/audio/search*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ source: null, preview_url: null, external_url: null, search_queries: [], uploader_name: null, spotify_uri: null }),
      });
    });

    await page.goto('/setlists/nonexistent');
    await expect(page.getByText(/not found/i)).toBeVisible();
    await expect(page.getByText('Back to Library')).toBeVisible();
  });
});
