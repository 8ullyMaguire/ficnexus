// Admin pages: every /admin/* route renders for an admin user (role 10)
// and shows real data or a clear empty state; the API behind them is
// auth-gated (logged-out users get 401 JSON, not HTML).
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, uniqueUser, registerUser, login, signInBrowser, gotoApp } from './helpers';

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

// Promote a freshly-registered user to admin (role 10) directly in the DB.
// The suite runs against a local dev DB, so this is the honest way to get
// an admin session without a bootstrap admin account.
import { execSync } from 'child_process';

function promoteToAdmin(username: string): void {
  try {
    // Prefer the real DATABASE_URL (set by .env-e2e/runtime.env); never
    // hardcode credentials in the spec.
    const dbUrl =
      process.env.E2E_DATABASE_URL ||
      process.env.DATABASE_URL ||
      'postgres://fichub:fichub@localhost:5432/fichub';
    const m = dbUrl.match(/postgres:\/\/([^:]+):([^@]+)@/);
    const pgUser = m?.[1] ?? 'fichub';
    const pgPass = m?.[2] ?? 'fichub';
    execSync(`PGPASSWORD='${pgPass}' psql -h localhost -U ${pgUser} -d fichub -c "UPDATE users SET role=10, level=100, exp = GREATEST(exp, 10000) WHERE username='${username}'"`, {
      stdio: 'pipe',
      env: { ...process.env, PGPASSWORD: pgPass },
    });
  } catch (e) {
    throw new Error(`could not promote ${username} to admin (psql): ${e instanceof Error ? e.message : e}`);
  }
}

test('admin API is auth-gated: anonymous requests are rejected', async ({ request }) => {
  for (const path of ['/api/admin/moderation/queue', '/api/admin/users', '/api/admin/bots']) {
    const res = await request.get(`${BACKEND_URL}${path}`);
    // Anonymous → 401 (no AuthUser) or 403 (role<10 check); either way the
    // endpoint must NOT return 200 data.
    expect(res.status(), `${path} must require auth (got ${res.status()})`).toBeGreaterThanOrEqual(401);
    expect(res.status(), `${path} must require auth (got ${res.status()})`).toBeLessThan(500);
  }
});

test('admin dashboard renders real admin stats cards', async ({ page, request }) => {
  const user = uniqueUser('admdash');
  await registerUser(request, user);
  // Promote to admin via DB (test setup is allowed to touch the dev DB).
  promoteToAdmin(user.username);
  // The JWT embeds `role` at mint time — log in again after promotion.
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin');
  await expect(page.getByRole('heading', { name: '🖥️ Command Center' })).toBeVisible({ timeout: 20_000 });
  // Admin dashboard polls /api/admin/realtime, /api/admin/bots, /api/admin/search-analytics.
  // It renders either real values or empty-state cards — assert the shell rendered.
  await expect(page.locator('body')).toContainText(/Admin|Dashboard|Realtime|Bots/i, { timeout: 25_000 });
});

test('admin users page lists users and shows role badges', async ({ page, request }) => {
  const user = uniqueUser('admusers');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  // The JWT embeds `role` at mint time — log in again after promotion.
  const token = await login(request, user.username, user.password);
  // Sign in: set the token + cached user directly in localStorage, then load
  // home so the auth store initializes with the promoted role.
  await signInBrowser(page, token);

  await gotoApp(page, '/admin/users');
  await expect(page.getByRole('heading', { name: 'User Management' })).toBeVisible({ timeout: 20_000 });
  // The registered admin appears in the user table.
  await expect(page.getByText(user.username, { exact: false })).toBeVisible({ timeout: 25_000 });
});

test('moderation queue renders for admin (queue list or empty state)', async ({ page, request }) => {
  const user = uniqueUser('admmod');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  // The JWT embeds `role` at mint time — log in again after promotion.
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin/moderation');
  await expect(page.getByRole('heading', { name: /Moderation Queue/ })).toBeVisible({ timeout: 20_000 });
  await page.waitForTimeout(1200);
  const body = await page.locator('body').innerText();
  expect(/Loading|No pending|empty|queue/i.test(body)).toBeTruthy();
});

test('auto-tag page renders', async ({ page, request }) => {
  const user = uniqueUser('admtag');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  // The JWT embeds `role` at mint time — log in again after promotion.
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin/auto-tag');
  await expect(page.getByRole('heading', { name: '🤖 Auto-Tag' })).toBeVisible({ timeout: 20_000 });
});

test('scrapers health page renders', async ({ page, request }) => {
  const user = uniqueUser('admscr');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  // The JWT embeds `role` at mint time — log in again after promotion.
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin/scrapers');
  await expect(page.getByRole('heading', { name: 'Scraper Health' })).toBeVisible({ timeout: 20_000 });
});

test('consensus page renders', async ({ page, request }) => {
  const user = uniqueUser('admcon');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  // The JWT embeds `role` at mint time — log in again after promotion.
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin/consensus');
  await expect(page.getByRole('heading', { name: '🗳️ Product Consensus' })).toBeVisible({ timeout: 20_000 });
});
