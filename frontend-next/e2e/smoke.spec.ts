import { test, expect } from '@playwright/test';

test.describe('Smoke tests', () => {
  test('home page loads with Tarab Studio heading', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Tarab Studio' })).toBeVisible();
  });

  test('home page shows navigation cards', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByText('Generate Setlist')).toBeVisible();
    await expect(page.getByText('Setlist Library')).toBeVisible();
    await expect(page.getByText('Crates')).toBeVisible();
    await expect(page.getByText('Track Catalog')).toBeVisible();
    await expect(page.getByText('Import from Spotify')).toBeVisible();
  });

  test('navbar is visible with brand and nav links', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav');
    await expect(nav.getByText('Tarab Studio')).toBeVisible();
    await expect(nav.getByText('Library')).toBeVisible();
    await expect(nav.getByText('Generate')).toBeVisible();
    await expect(nav.getByText('Crates')).toBeVisible();
    await expect(nav.getByText('Catalog')).toBeVisible();
    await expect(nav.getByText('Import')).toBeVisible();
  });

  test('clicking Library nav link navigates to /setlists', async ({ page }) => {
    await page.goto('/');
    await page.locator('nav').getByText('Library').click();
    await expect(page).toHaveURL('/setlists');
  });

  test('clicking a home card navigates to the correct route', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: /Generate Setlist/ }).click();
    await expect(page).toHaveURL('/setlist/generate');
  });
});
