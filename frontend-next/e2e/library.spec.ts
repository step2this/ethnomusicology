import { test, expect } from '@playwright/test';

const mockSetlists = [
  {
    id: 'set-1',
    name: 'Friday Night Set',
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

const mockCrates = [
  {
    id: 'crate-1',
    name: 'Weekend Set',
    track_count: 25,
    created_at: '2026-03-05T00:00:00Z',
    updated_at: '2026-03-07T00:00:00Z',
  },
  {
    id: 'crate-2',
    name: 'Chill Vibes',
    track_count: 10,
    created_at: '2026-03-04T00:00:00Z',
    updated_at: '2026-03-06T00:00:00Z',
  },
];

const mockTracks = {
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
    {
      id: 'track-2',
      title: 'Strings of Life',
      artist: 'Derrick May',
      bpm: 126,
      key: 'Am',
      energy: 0.75,
      duration_ms: 300000,
      preview_url: null,
      album_art_url: null,
      spotify_id: null,
      source_playlist_id: null,
      date_added: '2026-02-28T00:00:00Z',
    },
  ],
  page: 1,
  total_pages: 2,
  total: 30,
};

test.describe('Setlist library', () => {
  test('loads and displays setlist items', async ({ page }) => {
    await page.route('**/api/setlists', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ setlists: mockSetlists }),
      });
    });

    await page.goto('/setlists');
    await expect(page.getByText('My Setlists')).toBeVisible();
    await expect(page.getByText('Friday Night Set')).toBeVisible();
    await expect(page.getByText('Deep house vibes')).toBeVisible();
    await expect(page.getByText('12 tracks')).toBeVisible();
    // Second setlist has no name, shows "Untitled"
    await expect(page.getByText('Untitled')).toBeVisible();
    await expect(page.getByText('Peak time techno')).toBeVisible();
  });

  test('shows empty state when no setlists', async ({ page }) => {
    await page.route('**/api/setlists', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ setlists: [] }),
      });
    });

    await page.goto('/setlists');
    await expect(page.getByText('No saved setlists')).toBeVisible();
    await expect(page.getByText('Generate one to get started')).toBeVisible();
  });

  test('has Generate New button linking to /setlist/generate', async ({ page }) => {
    await page.route('**/api/setlists', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ setlists: mockSetlists }),
      });
    });

    await page.goto('/setlists');
    const generateButton = page.getByRole('link', { name: /Generate New/i });
    await expect(generateButton).toBeVisible();
    await expect(generateButton).toHaveAttribute('href', '/setlist/generate');
  });
});

test.describe('Crate library', () => {
  test('loads and displays crate items', async ({ page }) => {
    await page.route('**/api/crates', (route) => {
      if (route.request().method() === 'GET') {
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ crates: mockCrates }),
        });
      } else {
        route.continue();
      }
    });

    await page.goto('/crates');
    await expect(page.getByText('My Crates')).toBeVisible();
    await expect(page.getByText('Weekend Set')).toBeVisible();
    await expect(page.getByText('25 tracks')).toBeVisible();
    await expect(page.getByText('Chill Vibes')).toBeVisible();
    await expect(page.getByText('10 tracks')).toBeVisible();
  });

  test('shows empty state when no crates', async ({ page }) => {
    await page.route('**/api/crates', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ crates: [] }),
      });
    });

    await page.goto('/crates');
    await expect(page.getByText('No crates yet')).toBeVisible();
    await expect(page.getByText('Create one to organize your setlists')).toBeVisible();
  });

  test('has create crate form', async ({ page }) => {
    await page.route('**/api/crates', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ crates: [] }),
      });
    });

    await page.goto('/crates');
    await expect(page.getByPlaceholder('New crate name...')).toBeVisible();
    await expect(page.getByRole('button', { name: /Create/i })).toBeVisible();
  });
});

test.describe('Track catalog', () => {
  test('loads and displays tracks in a table', async ({ page }) => {
    await page.route('**/api/tracks*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockTracks),
      });
    });

    await page.goto('/tracks');
    await expect(page.getByText('Track Catalog')).toBeVisible();
    await expect(page.getByText('(30 tracks)')).toBeVisible();
    await expect(page.getByText('Starry Night')).toBeVisible();
    await expect(page.getByText('Peggy Gou')).toBeVisible();
    await expect(page.getByText('122')).toBeVisible();
    await expect(page.getByText('Strings of Life')).toBeVisible();
    await expect(page.getByText('Derrick May')).toBeVisible();
  });

  test('shows pagination controls for multi-page results', async ({ page }) => {
    await page.route('**/api/tracks*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockTracks),
      });
    });

    await page.goto('/tracks');
    await expect(page.getByText('Page 1 of 2')).toBeVisible();
    await expect(page.getByRole('button', { name: /Previous/i })).toBeDisabled();
    await expect(page.getByRole('button', { name: /Next/i })).toBeEnabled();
  });

  test('shows empty state when no tracks', async ({ page }) => {
    await page.route('**/api/tracks*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ tracks: [], page: 1, total_pages: 1, total: 0 }),
      });
    });

    await page.goto('/tracks');
    await expect(page.getByText('No tracks imported yet')).toBeVisible();
    await expect(page.getByRole('link', { name: /Import from Spotify/i })).toBeVisible();
  });

  test('table has sortable column headers', async ({ page }) => {
    await page.route('**/api/tracks*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockTracks),
      });
    });

    await page.goto('/tracks');
    // Column headers are buttons for sorting
    await expect(page.getByRole('button', { name: /Title/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Artist/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /BPM/i })).toBeVisible();
  });
});
