import { test, expect, Page } from '@playwright/test';

// Flutter renders to <canvas>, so Playwright can't see text in the DOM.
// Clicking the hidden "Enable accessibility" button activates Flutter's
// semantics layer, which overlays <flt-semantics> DOM elements that
// Playwright can query with getByText/getByRole.
async function enableFlutterAccessibility(page: Page) {
  // Flutter renders to <canvas>. Click hidden button to enable semantic DOM overlays.
  const a11yButton = page.getByRole('button', { name: 'Enable accessibility' });
  await a11yButton.click({ timeout: 15000 });
  await page.waitForTimeout(1000);
}

test.beforeAll(async ({ request }) => {
  const response = await request.post('/api/dev/seed');
  expect(response.ok()).toBeTruthy();
});

test.describe('SPA Routing', () => {
  test('direct navigation to /tracks serves app (not 404)', async ({ page }) => {
    await page.goto('/tracks');
    await enableFlutterAccessibility(page);
    // Flutter should bootstrap and GoRouter handles /tracks route
    // Either the app title or track data should be visible
    await expect(
      page.getByText('Salamic Vibes').or(page.getByText('Nour El Ain'))
    ).toBeVisible({ timeout: 15000 });
  });

  test('API routes still return JSON (not index.html)', async ({ request }) => {
    const response = await request.get('/api/health');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.status).toBe('ok');
  });

  test('dev seed endpoint is accessible', async ({ request }) => {
    const response = await request.post('/api/dev/seed');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.seeded).toBe(true);
  });
});
