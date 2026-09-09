// Lane 2 E2E — Report flow: TL1 reports a post → TL5 sees + resolves it in the queue.
// Requires the real backend on :8000 (E2E_BASE_URL) + a reachable dev DB.
// Skipped loudly when either is unavailable.
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

test.beforeEach(async ({}, testInfo) => {
  test.skip(!backendUp, `Skipping ${testInfo.title} — backend not up on ${BACKEND_URL} (${backendReason})`);
});

// Promote a user to a specific trust level directly in the DB.
function promoteToTrustLevel(username: string, trustLevel: number): void {
  try {
    const dbUrl =
      process.env.E2E_DATABASE_URL ||
      process.env.DATABASE_URL ||
      'postgres://fichub:***@localhost:5432/fichub';
    const m = dbUrl.match(/postgres:\/\/([^:]+):([^@]+)@/);
    const pgUser = m?.[1] ?? 'fichub';
    const pgPass = m?.[2] ?? 'fichub';
    execSync(
      `PGPASSWORD=${pgPass} psql -h localhost -U ${pgUser} -d fichub -c "UPDATE users SET trust_level = ${trustLevel} WHERE username = '${username}';"`,
      { stdio: 'pipe' },
    );
  } catch (e) {
    console.warn(`Failed to promote ${username} to TL${trustLevel}:`, e);
  }
}

// Create a test work and return its ID.
async function seedWork(request: ReturnType<typeof pwRequest.newContext>, token: string): Promise<number> {
  const content = 'Chapter 1\nA test story for the report flow.\n';
  const res = await request.post(`${BACKEND_URL}/api/upload`, {
    headers: { Authorization: `Bearer ${token}` },
    multipart: {
      title: 'Report Flow Test Work',
      author: 'Test Author',
      summary: 'A test work',
      tags: 'test, report',
      status: 'ongoing',
      file: {
        name: 'story.txt',
        mimeType: 'text/plain',
        buffer: Buffer.from(content, 'utf8'),
      },
    },
  });
  expect(res.ok(), `upload should succeed (got ${res.status()})`).toBeTruthy();
  const body = (await res.json()) as { err: number; url_id?: string };
  expect(body.err).toBe(0);
  return Number(body.url_id);
}

// Create a forum post and return its ID.
async function createForumPost(
  request: ReturnType<typeof pwRequest.newContext>,
  token: string,
  topicId: number,
  body: string,
): Promise<number> {
  const res = await request.post(`${BACKEND_URL}/api/forum/topics/${topicId}/posts`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { body },
  });
  expect(res.ok(), `create post should succeed (got ${res.status()})`).toBeTruthy();
  const json = (await res.json()) as { err: number; id?: number };
  expect(json.err).toBe(0);
  return Number(json.id);
}

// Create a forum topic and return its ID.
async function createForumTopic(
  request: ReturnType<typeof pwRequest.newContext>,
  token: string,
  categoryId: number,
  title: string,
): Promise<number> {
  const res = await request.post(`${BACKEND_URL}/api/forum/topics`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { category_id: categoryId, title, body: 'Test topic body' },
  });
  expect(res.ok(), `create topic should succeed (got ${res.status()})`).toBeTruthy();
  const json = (await res.json()) as { err: number; id?: number };
  expect(json.err).toBe(0);
  return Number(json.id);
}

// Create a forum category (admin only).
async function createForumCategory(
  request: ReturnType<typeof pwRequest.newContext>,
  token: string,
  name: string,
): Promise<number> {
  const res = await request.post(`${BACKEND_URL}/api/forum/categories`, {
    headers: { Authorization: `Bearer ${token}` },
    data: { name, slug: name.toLowerCase().replace(/\s+/g, '-'), description: 'Test category' },
  });
  expect(res.ok(), `create category should succeed (got ${res.status()})`).toBeTruthy();
  const json = (await res.json()) as { err: number; id?: number };
  expect(json.err).toBe(0);
  return Number(json.id);
}

test('TL1 reports a post → TL5 sees + resolves it in the queue', async ({ page, request }) => {
  // 1. Set up users
  const reporter = uniqueUser('reporter');
  const moderator = uniqueUser('moderator');

  const reporterToken = await registerUser(request, reporter);
  const moderatorToken = await registerUser(request, moderator);

  // Promote reporter to TL1 (can report)
  promoteToTrustLevel(reporter.username, 1);
  // Promote moderator to TL5 (can moderate)
  promoteToTrustLevel(moderator.username, 5);

  // Re-login to get fresh JWTs with updated trust levels
  const reporterTokenFresh = await login(request, reporter.username, reporter.password);
  const moderatorTokenFresh = await login(request, moderator.username, moderator.password);

  // 2. Moderator creates a category and topic
  const categoryId = await createForumCategory(request, moderatorTokenFresh, 'Report Test Category');
  const topicId = await createForumTopic(request, moderatorTokenFresh, categoryId, 'Report Test Topic');

  // 3. Reporter creates a post (to be reported)
  const postId = await createForumPost(request, reporterTokenFresh, topicId, 'This is a test post that will be reported.');

  // 4. Reporter reports the post
  const reportRes = await request.post(`${BACKEND_URL}/api/reports`, {
    headers: { Authorization: `Bearer ${reporterTokenFresh}` },
    data: {
      target_type: 'forum_post',
      target_id: postId,
      reason: 'This post violates community guidelines',
      category: 'spam',
    },
  });
  expect(reportRes.ok(), `report should succeed (got ${reportRes.status()})`).toBeTruthy();
  const reportBody = (await reportRes.json()) as { err: number; report_id?: number };
  expect(reportBody.err).toBe(0);
  const reportId = Number(reportBody.report_id);

  // 5. Reporter checks their report status
  const statusRes = await request.get(`${BACKEND_URL}/api/reports/${reportId}/status`, {
    headers: { Authorization: `Bearer ${reporterTokenFresh}` },
  });
  expect(statusRes.ok(), `status check should succeed (got ${statusRes.status()})`).toBeTruthy();
  const statusBody = (await statusRes.json()) as { err: number; status?: string };
  expect(statusBody.err).toBe(0);
  expect(statusBody.status).toBe('pending');

  // 6. Moderator views the report queue
  const queueRes = await request.get(`${BACKEND_URL}/api/admin/reports?status=pending`, {
    headers: { Authorization: `Bearer ${moderatorTokenFresh}` },
  });
  expect(queueRes.ok(), `queue view should succeed (got ${queueRes.status()})`).toBeTruthy();
  const queueBody = (await queueRes.json()) as { err: number; items?: Array<{ id: number; status: string }> };
  expect(queueBody.err).toBe(0);
  const foundReport = queueBody.items?.find((r) => r.id === reportId);
  expect(foundReport, 'report should appear in moderator queue').toBeTruthy();
  expect(foundReport?.status).toBe('pending');

  // 7. Moderator resolves the report
  const resolveRes = await request.post(`${BACKEND_URL}/api/admin/reports/${reportId}/resolve`, {
    headers: { Authorization: `Bearer ${moderatorTokenFresh}` },
    data: { action: 'resolve', note: 'Reviewed — no action needed' },
  });
  expect(resolveRes.ok(), `resolve should succeed (got ${resolveRes.status()})`).toBeTruthy();
  const resolveBody = (await resolveRes.json()) as { err: number; status?: string };
  expect(resolveBody.err).toBe(0);
  expect(resolveBody.status).toBe('resolved');

  // 8. Reporter sees the updated status
  const finalStatusRes = await request.get(`${BACKEND_URL}/api/reports/${reportId}/status`, {
    headers: { Authorization: `Bearer ${reporterTokenFresh}` },
  });
  expect(finalStatusRes.ok()).toBeTruthy();
  const finalStatusBody = (await finalStatusRes.json()) as { err: number; status?: string };
  expect(finalStatusBody.err).toBe(0);
  expect(finalStatusBody.status).toBe('resolved');
});

test('TL1 cannot access admin report queue', async ({ request }) => {
  const user = uniqueUser('tl1user');
  const token = await registerUser(request, user);
  promoteToTrustLevel(user.username, 1);
  const freshToken = await login(request, user.username, user.password);

  const res = await request.get(`${BACKEND_URL}/api/admin/reports`, {
    headers: { Authorization: `Bearer ${freshToken}` },
  });
  expect(res.status()).toBe(403);
});

test('reporter cannot see other users reports', async ({ request }) => {
  const reporter1 = uniqueUser('reporter1');
  const reporter2 = uniqueUser('reporter2');
  const mod = uniqueUser('mod2');

  const r1Token = await registerUser(request, reporter1);
  const r2Token = await registerUser(request, reporter2);
  const modToken = await registerUser(request, mod);

  promoteToTrustLevel(reporter1.username, 1);
  promoteToTrustLevel(reporter2.username, 1);
  promoteToTrustLevel(mod.username, 5);

  const r1Fresh = await login(request, reporter1.username, reporter1.password);
  const r2Fresh = await login(request, reporter2.username, reporter2.password);
  const modFresh = await login(request, mod.username, mod.password);

  // Mod creates content
  const catId = await createForumCategory(request, modFresh, 'Privacy Test Cat');
  const topicId = await createForumTopic(request, modFresh, catId, 'Privacy Test Topic');
  const postId = await createForumPost(request, r1Fresh, topicId, 'Post for privacy test');

  // Reporter1 reports
  const reportRes = await request.post(`${BACKEND_URL}/api/reports`, {
    headers: { Authorization: `Bearer ${r1Fresh}` },
    data: { target_type: 'forum_post', target_id: postId, reason: 'Privacy test', category: 'other' },
  });
  const reportBody = (await reportRes.json()) as { err: number; report_id?: number };
  const reportId = Number(reportBody.report_id);

  // Reporter2 tries to see Reporter1's report status
  const statusRes = await request.get(`${BACKEND_URL}/api/reports/${reportId}/status`, {
    headers: { Authorization: `Bearer ${r2Fresh}` },
  });
  expect(statusRes.status()).toBe(403);
});
