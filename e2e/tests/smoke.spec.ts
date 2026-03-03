import { test, expect, Page } from '@playwright/test';

// Flutter renders to <canvas>, so Playwright can't see text in the DOM.
// Clicking the hidden "Enable accessibility" button activates Flutter's
// semantics layer, which overlays <flt-semantics> DOM elements that
// Playwright can query with getByText/getByRole.
async function enableFlutterAccessibility(page: Page) {
  // Flutter renders to <canvas>. Activate semantic DOM overlays by dispatching click
  // on the hidden accessibility button. dispatchEvent bypasses positioning checks —
  // needed because Flutter renders this element with zero dimensions offscreen.
  const a11yButton = page.getByRole('button', { name: 'Enable accessibility' });
  await a11yButton.waitFor({ timeout: 15000 });
  await a11yButton.dispatchEvent('click');
  await page.waitForTimeout(1000);
}

// Seed test data before all tests in this file
test.beforeAll(async ({ request }) => {
  const response = await request.post('/api/dev/seed');
  expect(response.ok()).toBeTruthy();
  const body = await response.json();
  expect(body.seeded).toBe(true);
  expect(body.tracks).toBe(8);
});

test.describe('Smoke Tests', () => {
  test('home screen loads with title', async ({ page }) => {
    await page.goto('/');
    await enableFlutterAccessibility(page);
    await expect(page.getByRole('heading', { name: 'Salamic Vibes' })).toBeVisible({ timeout: 15000 });
  });

  test('track catalog shows seeded data', async ({ page }) => {
    await page.goto('/');
    await enableFlutterAccessibility(page);
    await expect(page.getByRole('heading', { name: 'Salamic Vibes' })).toBeVisible({ timeout: 15000 });

    // Click Track Catalog button
    await page.getByText('Track Catalog').click();

    // Wait for seeded track to appear (proves Frontend → API → DB round-trip)
    await expect(page.getByText('Nour El Ain')).toBeVisible({ timeout: 10000 });
  });

  test('generate setlist screen loads', async ({ page }) => {
    await page.goto('/');
    await enableFlutterAccessibility(page);
    await expect(page.getByRole('heading', { name: 'Salamic Vibes' })).toBeVisible({ timeout: 15000 });

    // Click Generate Setlist button
    await page.getByText('Generate Setlist').click();

    // Assert setlist generation screen has the vibe tab (ST-006 redesigned to tabbed input)
    await expect(page.getByText('Describe a Vibe')).toBeVisible({ timeout: 10000 });
  });
});
