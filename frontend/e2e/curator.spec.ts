// Curator pages: /curator/authors loads for a curator/admin and renders
// pending author-merge proposals (or the "nothing pending" state). The
// page redirects logged-out users home (auth guard), so we also assert
// that redirect behavior.
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

import { execSync } from 'child_process';

function promoteToCurator(username: string): void {
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
    throw new Error(`could not promote ${username} to curator (psql): ${e instanceof Error ? e.message : e}`);
  }
}

test('curator authors page loads and renders pending merges (or empty state)', async ({ page, request }) => {
  const user = uniqueUser('cur');
  await registerUser(request, user);
  promoteToCurator(user.username);
  // The JWT embeds `role` at mint time — log in AGAIN after promotion so
  // the token carries role 10 (the page's isAdmin gate reads the claim).
  const token = await login(request, user.username, user.password);
  await signInBrowser(page, token);

  await gotoApp(page, '/curator/authors');
  // The page heading is the author-merge queue title.
  await expect(page.getByRole('heading', { name: '🧬 Author Merge Queue' })).toBeVisible({ timeout: 20_000 });

  // Either a list of proposals or the "nothing pending" empty state.
  const body = await page.locator('body').innerText();
  const hasProposals = /proposal/i.test(body) && /approve|merge/i.test(body);
  const hasEmpty = body.includes('No pending merge proposals');
  expect(hasProposals || hasEmpty, `curator page should render proposals or empty state`).toBeTruthy();
});

test('curator page redirects logged-out users home (auth guard)', async ({ page }) => {
  await gotoApp(page, '/curator/authors');
  await page.waitForURL('**/', { timeout: 15_000 });
  await expect(page.getByRole('button', { name: 'Login' })).toBeVisible();
});
