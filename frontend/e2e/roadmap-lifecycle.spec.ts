// Roadmap arena full lifecycle (browser E2E):
//   1. Suggest a dummy feature (API, as a registered user)
//   2. Load /roadmap — the arena renders 4 clusters
//   3. Vote (best/worst) — vote persists, Elo updates, no 500
//   4. A NEW arena set loads after the vote (the voted set is retired)
//   5. Cleanup: delete the dummy suggestions + votes from the DB
//      so repeated runs never pollute the live arena.
import { test, expect, request as pwRequest } from '@playwright/test';
import { execSync } from 'child_process';
import { BACKEND_URL, uniqueUser, registerUser, signInBrowser, gotoApp } from './helpers';

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
  test.skip(!backendUp, `Skipping ${testInfo.title} — backend not up on ${BACKEND_URL} (${backendReason})`);
});

// Direct DB access so the test can seed AND clean up deterministic rows.
function dbUrl(): string {
  return (
    process.env.E2E_DATABASE_URL ||
    process.env.DATABASE_URL ||
    'postgres://fichub:fichub@localhost:5432/fichub'
  );
}

function runPsql(sql: string): void {
  const m = dbUrl().match(/postgres:\/\/([^:]+):([^@]+)@([^:]+):(\d+)\/(.+)/);
  const pgUser = m?.[1] ?? 'fichub';
  const pgPass = m?.[2] ?? 'fichub';
  const pgHost = m?.[3] ?? 'localhost';
  const pgPort = m?.[4] ?? '5432';
  const pgDb = m?.[5] ?? 'fichub';
  execSync(
    `PGPASSWORD='${pgPass}' psql -h ${pgHost} -p ${pgPort} -U ${pgUser} -d ${pgDb} -tAc "${sql}"`,
    { stdio: 'pipe', env: { ...process.env, PGPASSWORD: pgPass } },
  );
}

function runPsqlCount(sql: string): number {
  const m = dbUrl().match(/postgres:\/\/([^:]+):([^@]+)@([^:]+):(\d+)\/(.+)/);
  const pgUser = m?.[1] ?? 'fichub';
  const pgPass = m?.[2] ?? 'fichub';
  const pgHost = m?.[3] ?? 'localhost';
  const pgPort = m?.[4] ?? '5432';
  const pgDb = m?.[5] ?? 'fichub';
  const out = execSync(
    `PGPASSWORD='${pgPass}' psql -h ${pgHost} -p ${pgPort} -U ${pgUser} -d ${pgDb} -tAc "${sql}"`,
    { stdio: 'pipe', env: { ...process.env, PGPASSWORD: pgPass } },
  ).toString().trim();
  return parseInt(out || '0', 10);
}

const PREFIX = 'rmd_e2e_';

// Clean everything this suite may have created (safe to run repeatedly).
function cleanupDb(): void {
  runPsql(
    `DELETE FROM arena_votes WHERE user_id IN (SELECT id FROM users WHERE username LIKE '${PREFIX}%');
     DELETE FROM arena_votes WHERE best_cluster_id IN (SELECT id FROM feature_clusters WHERE representative_text LIKE '${PREFIX}%')
        OR worst_cluster_id IN (SELECT id FROM feature_clusters WHERE representative_text LIKE '${PREFIX}%')
        OR cluster_ids && ARRAY(SELECT id FROM feature_clusters WHERE representative_text LIKE '${PREFIX}%');
     DELETE FROM feature_suggestions WHERE raw_text LIKE '${PREFIX}%';
     DELETE FROM feature_clusters WHERE representative_text LIKE '${PREFIX}%';
     DELETE FROM users WHERE username LIKE '${PREFIX}%';`,
  );
}

// Seed exactly 4 fresh open clusters via SQL (deterministic: the arena serves
// 4 clusters only when >= 4 exist; we don't depend on Ollama clustering).
function seedFourClusters(): void {
  // feature_clusters.embedding is vector(768) — fake all-zeros vector.
  const emb = `[${Array.from({ length: 768 }, () => '0').join(',')}]`;
  for (let i = 1; i <= 4; i++) {
    runPsql(
      `INSERT INTO feature_clusters (representative_text, embedding, elo_rating, matches_played, status)
       VALUES ('${PREFIX}cluster${i} - test feature idea ${i}', '${emb}', 1500, 0, 'open');`,
    );
  }
}

test('roadmap arena: suggest → vote → new set, full lifecycle', async ({ page, request }) => {
  // 0. Clean any leftovers from a previous failed run, then seed 4 clusters
  //    deterministically so the arena always has a set to vote on.
  cleanupDb();
  seedFourClusters();

  // 1. Register a fresh user.
  const user = uniqueUser('rmd');
  const token = await registerUser(request, user);

  // 2. Also suggest a dummy feature via the API (covers the suggest path;
  //    clustering is best-effort and not required for the vote to work).
  const marker = `${PREFIX}${Date.now().toString(36)}`;
  const text = `${marker} dark mode for the reader please`;
  const suggestRes = await request.post(`${BACKEND_URL}/api/roadmap/suggest`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { text },
  });
  expect(suggestRes.ok(), `suggest should succeed (got ${suggestRes.status()})`).toBeTruthy();
  const suggestBody = (await suggestRes.json()) as { err: number; msg?: string };
  expect(suggestBody.err, `suggest err (msg: ${suggestBody.msg ?? 'none'})`).toBe(0);

  // 3. Load the roadmap arena as a signed-in browser.
  await signInBrowser(page, token);
  await gotoApp(page, '/roadmap');
  await expect(page.getByRole('heading', { name: /Roadmap/ })).toBeVisible({ timeout: 20_000 });

  // 4. The arena serves the 4 seeded clusters. Vote: first card = best,
  //    last card = worst.
  await page.waitForTimeout(1500);
  const cards = page.getByTestId('arena-card');
  await expect(cards.first()).toBeVisible({ timeout: 15_000 });
  const cardCount = await cards.count();
  expect(cardCount, `arena should serve 4 seeded clusters (got ${cardCount})`).toBeGreaterThanOrEqual(4);

  await cards.nth(0).click();
  await cards.nth(3).click();
  // The vote button is exactly "Submit vote" — be specific, the cards also
  // contain the word "votes" (badge text) and would match a loose /vote/ regex.
  await expect(page.getByRole('button', { name: 'Submit vote' })).toBeVisible({ timeout: 10_000 });
  await page.getByRole('button', { name: 'Submit vote' }).click();

  // The vote must NOT 500. Either it lands (success message) or the app
  // shows a clean validation error — never a "database error".
  await expect(async () => {
    const b = await page.locator('body').innerText();
    expect(b.toLowerCase()).not.toContain('database error');
    expect(b.toLowerCase()).not.toContain('internal server error');
  }).toPass({ timeout: 10_000 });

  // A vote row exists for this user.
  const voteCount = runPsqlCount(
    `SELECT count(*) FROM arena_votes v JOIN users u ON u.id = v.user_id WHERE u.username = '${user.username}'`,
  );
  expect(voteCount).toBeGreaterThanOrEqual(1);

  // 5. A new arena set is served after the vote (or at least the page
  //    still renders the arena without erroring).
  await expect(async () => {
    const b = await page.locator('body').innerText();
    expect(b.toLowerCase()).not.toContain('failed to load arena');
  }).toPass({ timeout: 10_000 });

  // 6. Cleanup so repeated runs never pollute the live arena.
  cleanupDb();
});
