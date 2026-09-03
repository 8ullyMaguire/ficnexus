// Roadmap arena lifecycle e2e: suggest → clean DB → arena → vote → Elo applied.
//
// Covers the full MaxDiff/Elo voting loop that previously hit a production
// FK violation for anonymous voters (user_id=0 → arena_votes_user_id_fkey).
// The vote handler now inserts NULL user_id for anonymous clients and keys
// per-client uniqueness on client_id.
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, uniqueUser, registerUser, gotoApp, signInBrowser } from './helpers';
import { execSync } from 'child_process';

let backendUp = false;
let backendReason = '';

function dbUrl(): string {
  return (
    process.env.E2E_DATABASE_URL ||
    process.env.DATABASE_URL ||
    'postgres://fichub:fichub@localhost:5432/fichub'
  );
}

function psql(sql: string): string {
  const m = dbUrl().match(/postgres:\/\/([^:]+):([^@]+)@/);
  const pgUser = m?.[1] ?? 'fichub';
  const pgPass = m?.[2] ?? 'fichub';
  try {
    return execSync(
      `PGPASSWORD='${pgPass}' psql -h localhost -U ${pgUser} -d fichub -tAc "${sql}"`,
      { stdio: 'pipe', env: { ...process.env, PGPASSWORD: pgPass } }
    ).toString().trim();
  } catch (e) {
    throw new Error(`psql failed: ${e instanceof Error ? e.message : e}`);
  }
}

// Remove everything this suite creates so reruns never collide. Order
// matters: arena_votes references feature_clusters, so drop votes first
// (by cluster membership, since client ids rotate per run), then clusters,
// then the raw suggestions.
function cleanupTestData(): void {
  psql(
    "DELETE FROM arena_votes WHERE best_cluster_id IN (SELECT id FROM feature_clusters WHERE representative_text LIKE 'E2E Arena %') OR worst_cluster_id IN (SELECT id FROM feature_clusters WHERE representative_text LIKE 'E2E Arena %'); " +
    "DELETE FROM feature_suggestions WHERE raw_text LIKE 'E2E Arena %'; " +
    "DELETE FROM feature_clusters WHERE representative_text LIKE 'E2E Arena %';"
  );
}

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

// Suggest a feature via the API (anonymous client, the path that previously
// broke). Returns the cluster id (0 if Ollama clustering didn't run).
async function suggestFeature(request: pwRequest.APIRequestContext, text: string, clientId: string): Promise<{ cluster_id: number; clustered: boolean }> {
  const res = await request.post('/api/roadmap/suggest', {
    headers: { 'content-type': 'application/json', 'x-client-id': clientId },
    data: { text },
  });
  const body = await res.json();
  expect(res.ok(), `suggest ${text}: ${JSON.stringify(body)}`).toBeTruthy();
  return { cluster_id: body.cluster_id ?? 0, clustered: body.clustered === true };
}

test('roadmap arena: anonymous suggest → arena → vote → Elo applied → next set', async ({ page, request }) => {
  cleanupTestData();
  const clientId = `e2e-arena-${Date.now()}`;

  // 1) Suggest 4 distinct dummy features so the arena has a set to show.
  //    Each gets a UNIQUE client id — the anti-spam limit is 3/day per
  //    identity, so a shared client id would block the 4th suggest.
  const texts = [
    'E2E Arena dark mode for the reader',
    'E2E Arena export to TXT format',
    'E2E Arena offline reading mode',
    'E2E Arena custom tag collections',
  ];
  const results: number[] = [];
  for (let i = 0; i < texts.length; i++) {
    const r = await suggestFeature(request, texts[i], `${clientId}-s${i}`);
    results.push(r.cluster_id);
  }
  const realClusters = results.filter((c) => c > 0);
  expect(realClusters.length, 'expected at least some clustered suggestions').toBeGreaterThanOrEqual(1);

  // 2) If clustering gave us < 4 clusters, insert a couple directly so the
  //    arena can present exactly 4 (test-only DB rows, cleaned up after).
  const count = realClusters.length;
  if (count < 4) {
    for (let i = count; i < 4; i++) {
      const emb = Array(768).fill('0.01').join(',');
      psql(`INSERT INTO feature_clusters (representative_text, embedding) VALUES ('E2E Arena filler ${i}', '[${emb}]'::vector);`);
    }
  }

  // 3) Load the arena page (signed-in browser so the vote is attributed to a
  //    real user id).
  const user = uniqueUser('arena');
  const token = await registerUser(request, user);
  await signInBrowser(page, token);
  await gotoApp(page, '/roadmap');
  await expect(page.getByRole('heading', { name: /Roadmap/ })).toBeVisible({ timeout: 20_000 });

  // The arena should render 4 cluster cards (we just seeded them).
  const arena = page.locator('.arena');
  await expect(async () => {
    const cards = await arena.locator('button.card, button.card.best, button.card.worst').count();
    expect(cards, `arena should render cards (got ${cards})`).toBeGreaterThanOrEqual(4);
  }).toPass({ timeout: 20_000 });

  // 4) Pick best + worst by clicking two arena cards, then submit.
  const cards = arena.locator('button.card, button.card.best, button.card.worst');
  await cards.nth(0).click();
  await cards.nth(1).click();
  await page.getByRole('button', { name: /submit vote/i }).click();

  // The vote should succeed. Assert no error text appears, then wait for the
  // post-vote reload to bring a new set (or the same set).
  await page.waitForTimeout(3_000);
  const bodyText = await page.locator('body').innerText();
  expect(/vote failed|database error/i.test(bodyText), `no vote error (body has: ${bodyText.slice(0, 200)})`).toBeFalsy();

  // 5) The vote row exists with this user's id, and the clusters' Elo moved.
  const voteCount = psql(
    `SELECT COUNT(*) FROM arena_votes v JOIN users u ON u.id = v.user_id WHERE u.username = '${user.username}'`
  );
  expect(parseInt(voteCount || '0', 10)).toBeGreaterThanOrEqual(1);

  const movedElo = psql(
    `SELECT COUNT(*) FROM feature_clusters WHERE representative_text LIKE 'E2E Arena %' AND (elo_rating::float8 != 1500 OR matches_played > 0)`
  );
  expect(parseInt(movedElo || '0', 10), 'at least one seeded cluster should have new Elo').toBeGreaterThanOrEqual(1);

  // 6) Anonymous vote path: a second client (no auth) can vote and the row
  //    stores NULL user_id (the exact regression this test guards).
  const anonReq = await pwRequest.newContext({ baseURL: BACKEND_URL });
  try {
    const arenaRes = await anonReq.get('/api/roadmap/arena', { headers: { 'x-client-id': clientId } });
    const arena = await arenaRes.json();
    expect(arena.err ?? 0, `arena for anon: ${JSON.stringify(arena)}`).toBe(0);
    if (arena.clusters && arena.clusters.length >= 4) {
      const ids = arena.clusters.map((c: { id: number }) => c.id);
      const voteRes = await anonReq.post('/api/roadmap/vote', {
        headers: { 'content-type': 'application/json', 'x-client-id': clientId },
        data: {
          cluster_ids: ids,
          best_cluster_id: ids[0],
          worst_cluster_id: ids[1],
        },
      });
      const vb = await voteRes.json();
      expect(voteRes.ok() && vb.err === 0, `anon vote should succeed: ${JSON.stringify(vb)}`).toBeTruthy();
      const nullUserVotes = psql(`SELECT COUNT(*) FROM arena_votes WHERE client_id = '${clientId}' AND user_id IS NULL`);
      expect(parseInt(nullUserVotes || '0', 10)).toBeGreaterThanOrEqual(1);
    }
  } finally {
    await anonReq.dispose();
  }

  // Cleanup after the test.
  cleanupTestData();
});
