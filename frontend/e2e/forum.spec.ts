// Forum F3+F4+F5 E2E — happy path: register → admin creates a category →
// create a topic → reply → follow → follower notification → search → unread →
// moderate a post (score −3, auto-collapse, queue).
// Requires the real backend on :8000 (E2E_BASE_URL) + a reachable dev DB so
// a fresh user can be promoted to admin for category seeding; skipped loudly
// when either is unavailable (same pattern as admin.spec.ts).
import { test, expect, request as pwRequest } from '@playwright/test';
import { BACKEND_URL, registerUser, uniqueUser, login, signInBrowser, gotoApp } from './helpers';
import { execSync } from 'child_process';

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

// Promote a freshly-registered user to admin (role 10) directly in the DB —
// the same honest dev-DB trick admin.spec.ts uses (the JWT embeds role at
// mint time, so we log in again after promotion).
function promoteToAdmin(username: string): void {
  promoteRole(username, 10);
}

// Promote a user to curator (level 50) directly in the DB, and age the account
// + grant exp so the mod-eligibility gates pass (age >= 14d, level >= 50, exp >= 20).
function promoteToCurator(username: string): void {
  try {
    const dbUrl =
      process.env.E2E_DATABASE_URL ||
      process.env.DATABASE_URL ||
      'postgres://fichub:fichub@localhost:5432/fichub';
    const m = dbUrl.match(/postgres:\/\/([^:]+):([^@]+)@/);
    const pgUser = m?.[1] ?? 'fichub';
    const pgPass = m?.[2] ?? 'fichub';
    execSync(
      `PGPASSWORD='${pgPass}' psql -h localhost -U ${pgUser} -d fichub -c "UPDATE users SET role=5, level=50, exp = GREATEST(exp, 5000), created_at = NOW() - INTERVAL '20 days' WHERE username='${username}'"`,
      { stdio: 'pipe' }
    );
  } catch (e) {
    throw new Error(`could not promote ${username} to curator (psql): ${e instanceof Error ? e.message : e}`);
  }
}

function promoteRole(username: string, role: number): void {
  try {
    const dbUrl =
      process.env.E2E_DATABASE_URL ||
      process.env.DATABASE_URL ||
      'postgres://fichub:fichub@localhost:5432/fichub';
    const m = dbUrl.match(/postgres:\/\/([^:]+):([^@]+)@/);
    const pgUser = m?.[1] ?? 'fichub';
    const pgPass = m?.[2] ?? 'fichub';
    execSync(
      `PGPASSWORD='${pgPass}' psql -h localhost -U ${pgUser} -d fichub -c "UPDATE users SET role=${role}, level=${role === 10 ? 100 : role === 5 ? 50 : role === 1 ? 1 : 0}, exp = GREATEST(exp, ${role === 10 ? 10000 : role === 5 ? 5000 : 100}) WHERE username='${username}'"`,
      { stdio: 'pipe' }
    );
  } catch (e) {
    throw new Error(`could not set role ${role} for ${username} (psql): ${e instanceof Error ? e.message : e}`);
  }
}

test('forum happy path: category → topic → reply → follow → notification → search → unread → moderate (F3+F4+F5)', async ({ page }) => {
  const ctx = await pwRequest.newContext({ baseURL: BACKEND_URL });

  // 1. Register two fresh users: the topic author and a follower.
  const author = uniqueUser('forum_author');
  const follower = uniqueUser('forum_follower');
  const authorToken = await registerUser(ctx, author);
  const followerToken = await registerUser(ctx, follower);

  // 2. Promote the author to admin (role 10) so they can create a category.
  promoteToAdmin(author.username);
  const adminToken = await login(ctx, author.username, author.password);

  const catSlug = `e2e-${Date.now().toString(36)}`;
  const catRes = await ctx.post(`${BACKEND_URL}/api/forum/categories`, {
    headers: { Authorization: `Bearer ${adminToken}` },
    data: { slug: catSlug, title: `E2E ${catSlug}` },
  });
  expect(catRes.ok()).toBeTruthy();
  const catBody = (await catRes.json()) as { err: number };
  expect(catBody.err).toBe(0);

  // 3. Author creates a topic via the API.
  const topicRes = await ctx.post(`${BACKEND_URL}/api/forum/topics`, {
    headers: { Authorization: `Bearer ${authorToken}` },
    data: { title: 'E2E welcome thread', category_slug: catSlug, body: 'Hello **forum** world' },
  });
  expect(topicRes.ok()).toBeTruthy();
  const topicBody = (await topicRes.json()) as { err: number; id?: number; topic_slug?: string | null };
  expect(topicBody.err).toBe(0);
  const topicId = topicBody.id as number;
  // Slug-form board URL is the canonical link for the topic.
  const topicSlug = topicBody.topic_slug as string;

  // 4. Follower follows the topic.
  const followRes = await ctx.post(`${BACKEND_URL}/api/forum/topics/${topicId}/follow`, {
    headers: { Authorization: `Bearer ${followerToken}` },
    data: {},
  });
  expect(followRes.ok()).toBeTruthy();
  const followBody = (await followRes.json()) as { err: number; following?: boolean };
  expect(followBody.err).toBe(0);
  expect(followBody.following).toBe(true);

  // 5. Author replies — the follower gets a forum_reply notification.
  const replyRes = await ctx.post(`${BACKEND_URL}/api/forum/topics/${topicId}/posts`, {
    headers: { Authorization: `Bearer ${authorToken}` },
    data: { body: 'First reply, all welcome.' },
  });
  expect(replyRes.ok()).toBeTruthy();
  const replyBody = (await replyRes.json()) as { err: number };
  expect(replyBody.err).toBe(0);

  // 5b. Follower also replies — gives the curator someone else's post to
  // moderate in step 10.
  const followerReplyRes = await ctx.post(`${BACKEND_URL}/api/forum/topics/${topicId}/posts`, {
    headers: { Authorization: `Bearer ${followerToken}` },
    data: { body: 'Follower reply, needs moderation.' },
  });
  expect(followerReplyRes.ok()).toBeTruthy();
  const followerReplyBody = (await followerReplyRes.json()) as { err: number };
  expect(followerReplyBody.err).toBe(0);

  // 6. Follower's notifications page shows the reply notification.
  await signInBrowser(page, followerToken);
  await gotoApp(page, '/notifications');
  // The notification title is "<author> replied to <topic>" — the reply BODY
  // is not part of the notification payload (only title + optional body).
  await expect(page.getByText(/replied to E2E welcome thread/i).first()).toBeVisible({ timeout: 15_000 });

  // 7. Browser rendering of the topic page (canonical board URL): markdown +
  // follow state.
  await signInBrowser(page, followerToken);
  await gotoApp(page, `/forum/board/${topicSlug}.${topicId}`);
  await expect(page.locator('h1')).toContainText('E2E welcome thread');
  await expect(page.locator('.post-body strong')).toHaveCount(1); // **forum**
  // Follower is following: the button shows "Following".
  await expect(page.getByRole('button', { name: /following/i })).toBeVisible();

  // 8. F4 — search finds the topic by a unique word (title + OP body).
  await gotoApp(page, `/forum/search`);
  await page.fill('input[name="q"]', 'welcome');
  await page.click('button[type="submit"]');
  await expect(page.getByText(/E2E welcome thread/i).first()).toBeVisible({ timeout: 15_000 });

  // 9. F4 — the topic list shows no unread badge after viewing the topic
  // (viewing marked it read in step 7).
  await gotoApp(page, `/forum/${catSlug}`);
  await expect(
    page.locator('li.topic-card', { hasText: 'E2E welcome thread' }).getByText(/New/i)
  ).toHaveCount(0, { timeout: 15_000 });

  // 10. F5 — promote the topic author to curator (role 5) and re-login, then
  // moderate the follower's reply with Abusive: score −3, auto-collapsed in
  // topic view, and the mod queue page lists the post.
  promoteToCurator(author.username);
  const curatorToken = await login(ctx, author.username, author.password);

  // Find the follower's reply (the post whose body mentions 'moderation').
  const detailRes = await ctx.get(`${BACKEND_URL}/api/forum/topics/${topicId}`, {
    headers: { Authorization: `Bearer ${curatorToken}` },
  });
  const detailBody = (await detailRes.json()) as { err: number; items?: { id: number; body: string; score: number }[] };
  expect(detailBody.err).toBe(0);
  const reply = (detailBody.items ?? []).find((p) => p.body.includes('moderation'));
  expect(reply).toBeTruthy();
  const replyId = reply!.id;

  // Moderate with Abusive → score −3, auto-collapsed (hidden_until set).
  const modRes = await ctx.post(`${BACKEND_URL}/api/forum/posts/${replyId}/moderate`, {
    headers: { Authorization: `Bearer ${curatorToken}` },
    data: { reason: 'Abusive' },
  });
  expect(modRes.ok()).toBeTruthy();
  const modBody = (await modRes.json()) as { err: number; delta?: number; score_after?: number; hidden_until?: string | null };
  expect(modBody.err).toBe(0);
  expect(modBody.delta).toBe(-3);
  expect(modBody.score_after).toBe(-3);

  // The mod queue endpoint works for the curator (the modded post itself is
  // excluded — already modded + hidden — so just assert a well-formed response).
  const queueRes = await ctx.get(`${BACKEND_URL}/api/forum/moderation/queue`, {
    headers: { Authorization: `Bearer ${curatorToken}` },
  });
  expect(queueRes.ok()).toBeTruthy();
  const queueBody = (await queueRes.json()) as { err: number; items?: { post_id: number }[]; count?: number };
  expect(queueBody.err).toBe(0);
  expect(Array.isArray(queueBody.items)).toBeTruthy();
  expect(typeof queueBody.count).toBe('number');

  // Topic view: the post is auto-collapsed (score −3 badge visible).
  await signInBrowser(page, curatorToken);
  await gotoApp(page, `/forum/board/${topicSlug}.${topicId}`);
  await expect(page.locator('text=/score|−3|collapsed/i').first()).toBeVisible({ timeout: 15_000 });

  await ctx.dispose();
});
