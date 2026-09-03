// Home page + search: the home dashboard renders trending (real
// /api/trending data) and the search page executes a real query and
// renders result cards.
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, uniqueUser, registerUser, seedFic, gotoApp } from './helpers';

let backendUp = false;
let backendReason = '';

test.beforeAll(async () => {
  const req = await pwRequest.newContext({ baseURL: BACKEND_URL });
  try {
    const res = await req.get('/api/health?skip_redis=true');
    backendUp = res.ok();
    if (!backendUp) backendReason = `health returned ${res.status()}`;
  } catch (e) {
    backendReason = `backend unreachable: ${e instanceof Error ? e.message : e}`;
  } finally {
    await req.dispose();
  }
});

// Skip every test in this file (loudly) when the real backend is missing.
test.beforeEach(async ({}, testInfo) => {
  test.skip(!backendUp, `Skipping ${testInfo.title} — backend not up on ${BACKEND_URL} (${backendReason})`);
});

test('homepage renders the dashboard with trending fics and download input', async ({ page }) => {
  await gotoApp(page, '/');

  // Download tab is the default view (root layout).
  await expect(page.getByRole('heading', { name: '✨ Recommended for you' })).toBeVisible();
  await expect(page.getByRole('heading', { name: '🔥 Trending this week' })).toBeVisible();
  // The trending section either lists real fics or the seeded empty state.
  await expect(page.locator('.home-section', { hasText: 'Trending this week' })).toBeVisible();

  // Download input is the core action on the first screen.
  await expect(page.getByLabel('Story URL to download')).toBeVisible();
  await expect(page.getByRole('button', { name: /Download/ }).first()).toBeVisible();

  // Decluttered home: the Blind Date card and the Surprise me random-fic
  // button are gone (both live in the Discover menu now).
  await expect(page.getByText('Blind Date with a Fic')).toHaveCount(0);
  await expect(page.getByRole('button', { name: /Surprise me/ })).toHaveCount(0);
  // The Atom feed link moved to the footer.
  await expect(page.locator('footer a[href="/feed.xml"]')).toHaveCount(1);
});

test('search executes a query and renders result cards', async ({ page, request }) => {
  // Seed a distinctive fic so the search has something deterministic to find.
  const user = uniqueUser('srch');
  const token = await registerUser(request, user);
  const urlId = await seedFic(request, token, 'The Emerald Lighthouse Quest', 'E2E Search Author', 'A hero searches for the emerald lighthouse at the edge of the world.');
  expect(urlId).toBeTruthy();

  await gotoApp(page, '/search');
  await expect(page.getByRole('heading', { name: 'Advanced Search' })).toBeVisible();

  const q = 'emerald';
  await page.getByPlaceholder('Search titles, descriptions, tags…').fill(q);
  await page.getByRole('button', { name: '🔍 Search' }).click();

  // Real results rendered as cards with a link to the fic page.
  await expect(page.locator('.result-card').first()).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('.results-header')).toContainText(/result/i);
  await expect(page.locator('.result-card a').first()).toHaveAttribute('href', new RegExp(`/fic/${urlId}`));
});

test('search with a nonsense query shows the no-results empty state', async ({ page }) => {
  await gotoApp(page, '/search');
  await page.getByPlaceholder('Search titles, descriptions, tags…').fill('zzzzqqqqxxyy');
  await page.getByRole('button', { name: '🔍 Search' }).click();
  await expect(page.getByText('No results found. Try different filters.')).toBeVisible({ timeout: 20_000 });
});
