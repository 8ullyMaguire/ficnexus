// Archive UI mode E2E tests — verifies the AO3-style archive layout
// renders correctly, URL param overrides work, preference persistence,
// and search results render in archive mode.
//
// These tests drive the REAL backend — skip cleanly when it's down.
import { test, expect, request as pwRequest } from '@playwright/test';
import {
  BACKEND_URL,
  uniqueUser,
  registerUser,
  seedFic,
  gotoApp,
} from './helpers';

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

test.beforeEach(async ({}, testInfo) => {
  test.skip(
    !backendUp,
    `Skipping ${testInfo.title} — backend not up on ${BACKEND_URL} (${backendReason})`,
  );
});

// ── 1. Default visit renders archive chrome ─────────────────────────
test('default visit renders archive chrome (maroon header, FicHub text, nav links)', async ({
  page,
}) => {
  // Clear localStorage so default prefs apply (uiMode defaults to 'archive')
  await page.goto('/');
  await page.evaluate(() => localStorage.clear());
  await page.goto('/');
  await page.waitForLoadState('domcontentloaded');

  // Archive header is visible with the FicHub brand text
  const header = page.locator('.archive-header');
  await expect(header).toBeVisible({ timeout: 10_000 });

  const brand = page.locator('.archive-brand');
  await expect(brand).toContainText('FicHub');

  // Accent line is the maroon bar under the header
  await expect(page.locator('.archive-accent-line')).toBeVisible();

  // Primary nav links are present
  const nav = page.locator('.archive-nav');
  await expect(nav).toContainText('Search');
  await expect(nav).toContainText('Browse');
  await expect(nav).toContainText('Requests');
  await expect(nav).toContainText('Forum');
  await expect(nav).toContainText('Ask');
});

// ── 2. Footer "Switch interface" toggles to modern mode ────────────
test('switch link in footer toggles to modern mode', async ({ page }) => {
  // Ensure we start in archive mode
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });
  await page.goto('/');
  await page.waitForLoadState('domcontentloaded');

  // Verify archive mode is active
  await expect(page.locator('.archive-header')).toBeVisible({ timeout: 10_000 });

  // Click the "Switch interface" button in the footer
  const switchBtn = page.locator('.archive-switch-link');
  await expect(switchBtn).toBeVisible();
  await switchBtn.click();

  // After the reload, archive chrome should be gone
  await page.waitForLoadState('domcontentloaded');
  await expect(page.locator('.archive-header')).toHaveCount(0, {
    timeout: 10_000,
  });

  // The modern layout should have its own topbar
  await expect(page.locator('.topbar')).toBeVisible({ timeout: 10_000 });
});

// ── 3. ?ui=modern URL param overrides to modern mode ───────────────
test('?ui=modern URL param overrides to modern mode', async ({ page }) => {
  // Set prefs to archive first
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });

  // Navigate with ?ui=modern — should force modern layout
  await page.goto('/?ui=modern');
  await page.waitForLoadState('domcontentloaded');
  await expect(page.locator('.archive-header')).toHaveCount(0, {
    timeout: 10_000,
  });
  await expect(page.locator('.topbar')).toBeVisible({ timeout: 10_000 });

  // Confirm localStorage was updated
  const stored = await page.evaluate(() => {
    const raw = localStorage.getItem('fichub_prefs_v1');
    return raw ? JSON.parse(raw) : null;
  });
  expect(stored?.uiMode).toBe('modern');
});

// ── 4. Reload persists the uiMode setting ──────────────────────────
test('reload persists the uiMode setting', async ({ page }) => {
  // Set archive mode explicitly
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });
  await page.goto('/');
  await page.waitForLoadState('domcontentloaded');
  await expect(page.locator('.archive-header')).toBeVisible({ timeout: 10_000 });

  // Reload the page — archive mode should still be active
  await page.reload();
  await page.waitForLoadState('domcontentloaded');
  await expect(page.locator('.archive-header')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('.archive-brand')).toContainText('FicHub');
});

// ── 5. Archive search renders WorkBlurb components when results exist ─
test('archive search renders WorkBlurb components when results exist', async ({
  page,
  request,
}) => {
  // Seed a fic so search has something to find
  const user = uniqueUser('arc');
  const token = await registerUser(request, user);
  const urlId = await seedFic(
    request,
    token,
    'Archive E2E Testwork',
    'Archive Test Author',
    'A test work for archive mode E2E coverage.',
  );
  expect(urlId).toBeTruthy();

  // Switch to archive mode
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });

  // Navigate to search in archive mode
  await page.goto('/search');
  await page.waitForLoadState('domcontentloaded');
  await expect(page.locator('.archive-header')).toBeVisible({ timeout: 10_000 });

  // Use the archive search input
  const searchInput = page.locator(
    '.archive-search input[type="search"], .archive-search input[type="text"], input[placeholder*="Search"]',
  );
  await searchInput.first().fill('Archive E2E Testwork');
  // Submit the search form
  await page.keyboard.press('Enter');

  // Wait for results — WorkBlurb articles should appear
  const blurb = page.locator('article.work-blurb');
  await expect(blurb.first()).toBeVisible({ timeout: 20_000 });

  // Verify the result contains the seeded fic title
  await expect(blurb.first()).toContainText('Archive E2E Testwork');
});

// ── 6. /tags page renders tag categories in archive mode ────────────
test('/tags page renders tag categories in archive mode', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });
  await page.goto('/tags');
  await page.waitForLoadState('domcontentloaded');
  await expect(page.locator('.archive-header, .tag-index')).toBeVisible({
    timeout: 10_000,
  });

  // The heading "Browse Tags" should be present in archive mode
  await expect(page.locator('.tag-index-heading, h1')).toContainText('Browse Tags', {
    timeout: 10_000,
  });

  // Tag categories should be rendered (tag-category divs) once loading completes.
  // The page fetches tags in parallel from the backend, so we wait for either
  // categories or the empty state.
  await expect(
    page.locator('.tag-category').first(),
  ).toBeVisible({ timeout: 15_000 });

  // At least one category heading should contain a label
  const heading = page.locator('.tag-category-heading').first();
  await expect(heading).not.toBeEmpty();
});

// ── 7. /bookmarks page shows login prompt when not authenticated ─────
test('/bookmarks page shows login prompt when not authenticated', async ({
  page,
}) => {
  // Clear auth state and set archive mode
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.clear();
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });
  await page.goto('/bookmarks');
  await page.waitForLoadState('domcontentloaded');

  // Should see the archive header
  await expect(page.locator('.archive-header')).toBeVisible({ timeout: 10_000 });

  // Should show the login prompt when not authenticated
  await expect(page.locator('.archive-empty')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('.archive-empty')).toContainText('Log in');

  // Should have a link to /login
  const loginLink = page.locator('.archive-empty a[href="/login"]');
  await expect(loginLink).toBeVisible();
});

// ── 8. /authors page has search input in archive mode ───────────────
test('/authors page has search input in archive mode', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });
  await page.goto('/authors');
  await page.waitForLoadState('domcontentloaded');

  // Archive heading should be visible
  await expect(page.locator('.archive-heading, .archive-authors')).toBeVisible({
    timeout: 10_000,
  });

  // Search input should be present with the expected placeholder
  const searchInput = page.locator(
    '.archive-search-row input, input[placeholder*="Search author"]',
  );
  await expect(searchInput.first()).toBeVisible();
  await expect(searchInput.first()).toHaveAttribute(
    'placeholder',
    /search author/i,
  );
});

// ── 9. /settings page shows Interface Style section in archive mode ──
test('/settings page shows Interface Style section in archive mode', async ({
  page,
}) => {
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.setItem(
      'fichub_prefs_v1',
      JSON.stringify({ uiMode: 'archive' }),
    );
  });
  await page.goto('/settings');
  await page.waitForLoadState('domcontentloaded');

  // The archive fieldset with "Interface Style" legend should be visible
  const fieldset = page.locator('fieldset.archive-fieldset');
  await expect(fieldset).toBeVisible({ timeout: 10_000 });
  await expect(fieldset.locator('legend')).toContainText('Interface Style');

  // The radio buttons for archive/modern should be present
  const radios = fieldset.locator('input[type="radio"]');
  await expect(radios).toHaveCount(2);

  // Archive mode should be the currently selected one
  const archiveRadio = fieldset.locator('input[name="uimode"]').first();
  await expect(archiveRadio).toBeChecked();
});
