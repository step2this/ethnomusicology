import { test, expect } from '@playwright/test';

// Mock all API routes to prevent real backend calls
function mockApis(page: import('@playwright/test').Page) {
  return Promise.all([
    page.route('**/api/setlists', (route) => {
      if (route.request().method() === 'GET') {
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ setlists: [] }),
        });
      } else {
        route.continue();
      }
    }),
    page.route('**/api/setlists/*', (route) => {
      route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ error: { code: 'NOT_FOUND', message: 'Setlist not found' } }),
      });
    }),
    page.route('**/api/crates', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ crates: [] }),
      });
    }),
    page.route('**/api/crates/*', (route) => {
      route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ error: { code: 'NOT_FOUND', message: 'Crate not found' } }),
      });
    }),
    page.route('**/api/tracks*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ tracks: [], page: 1, total_pages: 1, total: 0 }),
      });
    }),
    page.route('**/api/auth/spotify/status*', (route) => {
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ connected: false }),
      });
    }),
  ]);
}

test.describe('Route navigation', () => {
  test('home page renders at /', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Tarab Studio' })).toBeVisible();
  });

  test('/setlists renders setlist library', async ({ page }) => {
    await mockApis(page);
    await page.goto('/setlists');
    await expect(page.getByText('My Setlists')).toBeVisible();
  });

  test('/setlists/test-id shows error for non-existent setlist', async ({ page }) => {
    await mockApis(page);
    await page.goto('/setlists/test-id');
    await expect(page.getByText('Setlist not found')).toBeVisible();
    await expect(page.getByText('Back to Library')).toBeVisible();
  });

  test('/crates renders crate library', async ({ page }) => {
    await mockApis(page);
    await page.goto('/crates');
    await expect(page.getByText('My Crates')).toBeVisible();
  });

  test('/crates/test-id shows error for non-existent crate', async ({ page }) => {
    await mockApis(page);
    await page.goto('/crates/test-id');
    await expect(page.getByText('Failed to load crate')).toBeVisible();
    await expect(page.getByText('Back to Crates')).toBeVisible();
  });

  test('/tracks renders track catalog', async ({ page }) => {
    await mockApis(page);
    await page.goto('/tracks');
    // Empty state shows "No tracks imported yet"
    await expect(page.getByText('No tracks imported yet')).toBeVisible();
  });

  test('/setlist/generate renders generation form', async ({ page }) => {
    await page.goto('/setlist/generate');
    await expect(page.getByRole('heading', { name: 'Generate Setlist' })).toBeVisible();
  });

  test('/import/spotify renders import page', async ({ page }) => {
    await mockApis(page);
    await page.goto('/import/spotify');
    await expect(page.getByText('Import from Spotify')).toBeVisible();
  });

  test('SPA navigation: nav link updates URL and content', async ({ page }) => {
    await mockApis(page);
    await page.goto('/');

    // Navigate to Library
    await page.locator('nav').getByText('Library').click();
    await expect(page).toHaveURL('/setlists');
    await expect(page.getByText('My Setlists')).toBeVisible();

    // Navigate to Crates
    await page.locator('nav').getByText('Crates').click();
    await expect(page).toHaveURL('/crates');
    await expect(page.getByText('My Crates')).toBeVisible();
  });

  test('browser back button works after SPA navigation', async ({ page }) => {
    await mockApis(page);
    await page.goto('/');
    await page.locator('nav').getByText('Library').click();
    await expect(page).toHaveURL('/setlists');

    await page.goBack();
    await expect(page).toHaveURL('/');
    await expect(page.getByRole('heading', { name: 'Tarab Studio' })).toBeVisible();
  });
});
