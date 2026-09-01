// Admin navigation regression: every link in the admin sidebar and on the
// Command Center dashboard must resolve to a real page that renders its
// heading. This is the guard for 'dead click' bugs — a link that navigates
// nowhere (missing route, routePages omission, 404) fails here.
//
// Pattern: one registered+promoted admin per test (the suite is serial,
// workers:1), then click the REAL sidebar link (not page.goto) so a broken
// href fails the click-through, and assert the target page's heading.
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
import { execSync } from 'child_process';

function promoteToAdmin(username: string): void {
  try {
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

// The sidebar link label → the heading that must render after clicking it.
// Add a new admin page to BOTH the sidebar in +layout.svelte and this map;
// the test is the regression guard for dead clicks.
const SIDEBAR_TARGETS: Array<{ link: string; href: string; heading: RegExp }> = [
  { link: 'Dashboard', href: '/admin', heading: /Command Center/ },
  { link: 'Auto-Tag', href: '/admin/auto-tag', heading: /Auto-Tag/ },
  { link: 'Consensus', href: '/admin/consensus', heading: /Product Consensus/ },
  { link: 'Bot Behavior', href: '/admin/bots', heading: /Bot Behavior/ },
  { link: 'Moderation', href: '/admin/moderation', heading: /Moderation Queue/ },
  { link: 'Comment Triage', href: '/admin/comment-triage', heading: /Comment Triage/ },
  { link: 'Stats', href: '/admin/stats', heading: /Platform Stats/ },
  { link: 'Scraper Health', href: '/admin/scrapers', heading: /Scraper Health/ },
  { link: 'Users', href: '/admin/users', heading: /User Management/ },
];

test('admin sidebar links: every link clicks through to a page that renders its heading', async ({ page, request }) => {
  const user = uniqueUser('admnava');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin');
  await expect(page.getByRole('heading', { name: '🖥️ Command Center' })).toBeVisible({ timeout: 20_000 });

  // 1. Every sidebar link must resolve to a real route (no dead hrefs).
  for (const t of SIDEBAR_TARGETS) {
    const link = page.locator('.admin-sidebar a', { hasText: t.link });
    await expect(link, `sidebar link ${t.link} should exist`).toHaveCount(1);
    const href = await link.getAttribute('href');
    expect(href, `sidebar link ${t.link} must have an href`).toBe(t.href);
  }

  // 2. Click each sidebar link and assert the target page's heading renders.
  //    This is the dead-click regression: a broken href or a route missing
  //    from routePages (layout falls back to the home dashboard) fails here.
  for (const t of SIDEBAR_TARGETS) {
    const link = page.locator('.admin-sidebar a', { hasText: t.link });
    await link.click();
    await expect(page, `clicking "${t.link}" should land on ${t.href}`).toHaveURL(new RegExp(t.href.replace('/', '\\/') + '$'));
    await expect(page.getByRole('heading', { name: t.heading }), `heading for ${t.href}`).toBeVisible({ timeout: 20_000 });
  }
});

test('admin dashboard quick links: Comment Triage, Platform Stats, Curator Flag Queue all resolve', async ({ page, request }) => {
  const user = uniqueUser('admquick');
  await registerUser(request, user);
  promoteToAdmin(user.username);
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/admin');
  await expect(page.getByRole('heading', { name: '🖥️ Command Center' })).toBeVisible({ timeout: 20_000 });

  const quickLinks = [
    { name: 'Comment Triage', href: '/admin/comment-triage', heading: /Comment Triage/ },
    { name: 'Platform Stats', href: '/admin/stats', heading: /Platform Stats/ },
    { name: 'Curator Flag Queue', href: '/curator/flags', heading: /Curator Flag Queue/ },
  ];
  for (const q of quickLinks) {
    const link = page.locator('.quick-links a', { hasText: q.name });
    await expect(link, `quick link ${q.name} should exist`).toHaveCount(1);
    expect(await link.getAttribute('href'), `quick link ${q.name} must point at ${q.href}`).toBe(q.href);
  }
  // Click-through for the admin quick links (flag queue is covered by curator.spec.ts).
  for (const q of quickLinks.slice(0, 2)) {
    await page.locator('.quick-links a', { hasText: q.name }).click();
    await expect(page.getByRole('heading', { name: q.heading })).toBeVisible({ timeout: 20_000 });
    // Back to the dashboard to click the next one.
    await gotoApp(page, '/admin');
    await expect(page.getByRole('heading', { name: '🖥️ Command Center' })).toBeVisible({ timeout: 20_000 });
  }
});

test('user dropdown shows Admin Dashboard + Curator links only for admins (role >= 10)', async ({ page, request }) => {
  // Non-admin: dropdown must NOT contain the admin links.
  const pleb = uniqueUser('admnavp');
  await registerUser(request, pleb);
  const plebToken = await login(request, pleb.username, pleb.password);
  await signInBrowser(page, plebToken);
  // The dropdown trigger is labelled with the username (icon + name).
  await page.getByRole('button', { name: new RegExp(pleb.username, 'i') }).click();
  await expect(page.getByRole('menu')).toBeVisible();
  await expect(page.getByRole('menu').getByRole('link', { name: /admin dashboard/i })).toHaveCount(0);
  await expect(page.getByRole('menu').getByRole('link', { name: /curator/i })).toHaveCount(0);

  // Admin (role 10): dropdown must show Admin Dashboard, Curator, Flag Queue.
  const admin = uniqueUser('admnavi');
  await registerUser(request, admin);
  promoteToAdmin(admin.username);
  const adminToken = await login(request, admin.username, admin.password);
  await signInBrowser(page, adminToken);
  await page.getByRole('button', { name: new RegExp(admin.username, 'i') }).click();
  await expect(page.getByRole('menu')).toBeVisible();
  await expect(page.getByRole('menu').getByRole('link', { name: /admin dashboard/i })).toHaveCount(1);
  await expect(page.getByRole('menu').getByRole('link', { name: /curator/i })).toHaveCount(1);
  await expect(page.getByRole('menu').getByRole('link', { name: /flag queue/i })).toHaveCount(1);
});
