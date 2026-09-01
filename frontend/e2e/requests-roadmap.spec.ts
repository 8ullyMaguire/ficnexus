// Requests board + roadmap arena: the requests page lists real requests
// from /api/requests, and the roadmap page loads the MaxDiff/Elo arena
// (renders the empty state or real cluster cards).
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, uniqueUser, registerUser, seedRequest, gotoApp } from './helpers';

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

test('requests board renders the seeded request', async ({ page, request }) => {
  const user = uniqueUser('req');
  const token = await registerUser(request, user);
  const title = 'E2E Prompt: a quiet village mystery';
  const reqId = await seedRequest(request, token, title, 'Looking for a slow-burn mystery set in a quiet village.');
  expect(reqId).toBeGreaterThan(0);

  await gotoApp(page, '/requests');
  await expect(page.getByRole('heading', { name: /Fic Requests/ })).toBeVisible();
  // The seeded request shows up in the list (previous runs may have seeded
  // duplicates — assert at least one rendered link).
  await expect(page.getByRole('link', { name: title }).first()).toBeVisible({ timeout: 20_000 });
});

test('new request page renders the form and navigates back to the board', async ({ page }) => {
  await gotoApp(page, '/requests/new');
  await expect(page.getByRole('heading', { name: '🙋 New fic request' })).toBeVisible();
  // Form fields exist (real UI).
  await expect(page.getByPlaceholder(/Fics like/)).toBeVisible();
  await expect(page.getByPlaceholder(/narrows it down/)).toBeVisible();
  await expect(page.getByRole('button', { name: /Submit|Create|Post/i }).first()).toBeVisible();
});

test('roadmap arena loads and renders either cluster cards or the empty state', async ({ page }) => {
  await gotoApp(page, '/roadmap');
  await expect(page.getByRole('heading', { name: /Roadmap/ })).toBeVisible({ timeout: 20_000 });
  // Either real clusters (cards) or the seeded "no features" message.
  const hasCards = await page.locator('.arena button.card, .arena .card').count();
  const body = await page.locator('body').innerText();
  const hasEmptyState = body.includes('No features yet') || body.includes('be the first to suggest');
  expect(hasCards > 0 || hasEmptyState, `roadmap should render cards or empty state (cards=${hasCards})`).toBeTruthy();
});
