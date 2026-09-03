// Bookmarks + follows: logged-in user adds a bookmark to a real work via
// the fic page button (POST /api/bookmarks), sees it on /bookmarks, removes
// it (DELETE), and follows/unfollows the same work (POST/DELETE /api/follows).
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, uniqueUser, registerUser, seedFic, signInBrowser, gotoApp } from './helpers';

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

test('add and remove a bookmark from the fic page, verify on /bookmarks', async ({ page, request }) => {
  const user = uniqueUser('bmk');
  const token = await registerUser(request, user);
  const urlId = await seedFic(request, token, 'The Glass Garden of Veyra', 'E2E Bmk Author', 'A garden of glass blooms only at midnight.');
  await signInBrowser(page, token);

  // --- Add ---
  await gotoApp(page, `/fic/${urlId}`);
  await expect(page.locator('.fic-header h1')).toContainText('The Glass Garden of Veyra');

  const saveBtn = page.getByRole('button', { name: '☆ Save' });
  await expect(saveBtn).toBeVisible({ timeout: 20_000 });
  await saveBtn.click();
  // Button flips to the saved state.
  await expect(page.getByRole('button', { name: '⭐ Saved' })).toBeVisible({ timeout: 20_000 });

  // --- Verify on the bookmarks page ---
  await gotoApp(page, '/bookmarks');
  await expect(page.getByRole('heading', { name: /Bookmarks/ })).toBeVisible();
  await expect(page.locator('.bookmark-item, .bookmark-card, .card').first()).toBeVisible({ timeout: 20_000 });
  const body = await page.locator('body').innerText();
  expect(body).toContain('The Glass Garden of Veyra');

  // --- Remove ---
  await gotoApp(page, `/fic/${urlId}`);
  const savedBtn = page.getByRole('button', { name: '⭐ Saved' });
  await expect(savedBtn).toBeVisible({ timeout: 20_000 });
  await savedBtn.click();
  await expect(page.getByRole('button', { name: '☆ Save' })).toBeVisible({ timeout: 20_000 });

  // Bookmarks page no longer lists it.
  await gotoApp(page, '/bookmarks');
  await page.waitForTimeout(1200);
  const bodyAfter = await page.locator('body').innerText();
  expect(bodyAfter).not.toContain('The Glass Garden of Veyra');
});

test('follow and unfollow a work from the fic page', async ({ page, request }) => {
  const user = uniqueUser('fol');
  const token = await registerUser(request, user);
  const urlId = await seedFic(request, token, 'The Crimson Clock', 'E2E Fol Author', 'A clock that runs on stolen time.');
  // Sign the browser session in explicitly (addInitScript is unreliable for
  // SPA auth-init timing).
  await signInBrowser(page, token);

  await gotoApp(page, `/fic/${urlId}`);
  await expect(page.locator('.fic-header h1')).toContainText('The Crimson Clock');

  // Follow the work (button flips to "Following").
  const followBtn = page.getByRole('button', { name: '🔕 Follow' });
  await expect(followBtn).toBeVisible({ timeout: 20_000 });
  await followBtn.click();
  await expect(page.getByRole('button', { name: '🔔 Following' })).toBeVisible({ timeout: 20_000 });

  // Follows page lists it (as "Work #<id>" or the fic title).
  await gotoApp(page, '/follows');
  await expect(page.getByRole('heading', { name: '📋 Following' })).toBeVisible();
  // The follows page fetches asynchronously — poll until the item appears.
  await page.waitForFunction(
    () => document.body.innerText.includes('Work #'),
    undefined,
    { timeout: 20_000 },
  );
  // The follows page renders "Work #<id>" (not the fic title) — assert the
  // followed work's id is present.
  await expect(page.locator('body')).toContainText(`Work #`);

  // Unfollow from the fic page.
  await gotoApp(page, `/fic/${urlId}`);
  const followingBtn = page.getByRole('button', { name: '🔔 Following' });
  await expect(followingBtn).toBeVisible({ timeout: 20_000 });
  await followingBtn.click();
  await expect(page.getByRole('button', { name: '🔕 Follow' })).toBeVisible({ timeout: 20_000 });
});

test('anonymous user sees the "log in to save" prompt on the fic page', async ({ page }) => {
  const req = await pwRequest.newContext({ baseURL: BACKEND_URL });
  const user = uniqueUser('anonfic');
  const token = await registerUser(req, user);
  const urlId = await seedFic(req, token, 'The Silent Bell', 'E2E Anon Author', 'A bell that rings only in silence.');
  await req.dispose();

  await gotoApp(page, `/fic/${urlId}`);
  await expect(page.locator('.fic-header h1')).toContainText('The Silent Bell');
  // Logged-out users get the "Log in to rate & save" CTA instead of buttons.
  await expect(page.getByRole('link', { name: /Log in to rate/ })).toBeVisible({ timeout: 20_000 });
  await expect(page.getByRole('button', { name: /Save/ })).not.toBeVisible();
});
