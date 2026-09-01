// F4 forum API client tests: markTopicRead + searchForum (mock fetch).

import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('markTopicRead', () => {
  it('POSTs /api/forum/topics/{id}/read with last_read_post_id', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, last_read_post_id: 52, updated_at: '2026-08-14T10:00:00Z' }));
    const { markTopicRead } = await import('./forum');
    const res = await markTopicRead(5, 52);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/topics/5/read');
    const init = mockFetch.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ last_read_post_id: 52 });
    expect(res.err).toBe(0);
    expect(res.last_read_post_id).toBe(52);
  });

  it('omits last_read_post_id when not provided (backend defaults to topic last_post_id)', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, last_read_post_id: 60, updated_at: '2026-08-14T10:00:00Z' }));
    const { markTopicRead } = await import('./forum');
    await markTopicRead(5);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/topics/5/read');
    expect(JSON.parse(String((mockFetch.mock.calls[0][1] as RequestInit).body))).toEqual({});
  });

  it('omits last_read_post_id when null is passed', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, last_read_post_id: null, updated_at: '2026-08-14T10:00:00Z' }));
    const { markTopicRead } = await import('./forum');
    await markTopicRead(5, null);
    expect(JSON.parse(String((mockFetch.mock.calls[0][1] as RequestInit).body))).toEqual({});
  });
});

describe('searchForum', () => {
  it('GETs /api/forum/search?q=... and returns results with snippets', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        q: 'dragon',
        results: [
          {
            type: 'topic',
            topic_id: 5,
            post_id: null,
            title: 'Dragon lore',
            author_id: 1,
            author_username: 'tester',
            body: 'All about dragons',
            snippet: 'All about <mark>dragon</mark>s',
            category_slug: 'general',
            category_title: 'General',
            created_at: '2026-08-13T10:00:00Z',
          },
          {
            type: 'post',
            topic_id: 5,
            post_id: 52,
            title: 'Dragon lore',
            author_id: 2,
            author_username: 'mod',
            body: 'A reply',
            snippet: 'A <mark>dragon</mark> reply',
            category_slug: 'general',
            category_title: 'General',
            created_at: '2026-08-13T11:00:00Z',
          },
        ],
        next_cursor: null,
        limit: 25,
        total: 2,
      }),
    );
    const { searchForum } = await import('./forum');
    const res = await searchForum('dragon');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/search?q=dragon');
    expect(res.err).toBe(0);
    expect(res.total).toBe(2);
    expect(res.results).toHaveLength(2);
    expect(res.results[0].type).toBe('topic');
    expect(res.results[0].post_id).toBeNull();
    expect(res.results[1].type).toBe('post');
    expect(res.results[1].snippet).toContain('<mark>');
  });

  it('appends category and limit query params', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, q: 'dragon', results: [], next_cursor: null, limit: 10, total: 0 }));
    const { searchForum } = await import('./forum');
    await searchForum('dragon', 'general', 10);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/search?q=dragon&category=general&limit=10');
  });

  it('skips category when null/empty and encodes the query', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, q: '', results: [], next_cursor: null, limit: 25, total: 0 }));
    const { searchForum } = await import('./forum');
    await searchForum('two words', null);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/search?q=two+words');
  });
});

describe('getForumTopicBySlug', () => {
  it('GETs /api/forum/topics/by-slug/{slug} and returns the detail', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        id: 5,
        title: 'The great debate',
        topic_slug: 'the-great-debate-5',
        author_id: 1,
        author_username: 'tester',
        category_slug: 'general',
        category_title: 'General',
        status: 'open',
        body: '**Hello** world',
        payload: null,
        view_count: 42,
        created_at: '2026-08-14T10:00:00Z',
        updated_at: '2026-08-14T10:00:00Z',
        items: [],
        next_cursor: null,
        limit: 25,
        view_count_before: 41,
      }),
    );
    const { getForumTopicBySlug } = await import('./forum');
    const res = await getForumTopicBySlug('the-great-debate-5');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/topics/by-slug/the-great-debate-5');
    expect(res.err).toBe(0);
    expect(res.id).toBe(5);
    expect(res.topic_slug).toBe('the-great-debate-5');
  });

  it('appends after/limit query params and encodes the slug', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 5, title: 'x', items: [], next_cursor: null, limit: 10, view_count_before: 1 }));
    const { getForumTopicBySlug } = await import('./forum');
    await getForumTopicBySlug('weird slug/with+chars', 100, 10);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/topics/by-slug/weird%20slug%2Fwith%2Bchars?after=100&limit=10');
  });
});

describe('F5 moderation API client', () => {
  it('MODERATION_REASONS lists the nine canonical reasons with deltas and label keys', async () => {
    const { MODERATION_REASONS } = await import('./forum');
    expect(MODERATION_REASONS).toHaveLength(9);
    expect(MODERATION_REASONS[0]).toEqual({ key: 'insightful', delta: 2, labelKey: 'forum.reason.insightful' });
    const byKey = Object.fromEntries(MODERATION_REASONS.map((r) => [r.key, r.delta]));
    expect(byKey).toEqual({
      insightful: 2,
      informative: 1,
      interesting: 1,
      funny: 1,
      'off-topic': -1,
      redundant: -1,
      flamebait: -2,
      troll: -2,
      abusive: -3,
    });
    for (const r of MODERATION_REASONS) {
      expect(r.labelKey).toBe(`forum.reason.${r.key}`);
    }
  });

  it('getModerationStatus GETs /api/forum/moderation/status', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, points_left: 3, expires_at: null, eligible: true }));
    const { getModerationStatus } = await import('./forum');
    const res = await getModerationStatus();
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/moderation/status');
    expect(res.points_left).toBe(3);
    expect(res.eligible).toBe(true);
  });

  it('getModerationQueue GETs /api/forum/moderation/queue', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        count: 1,
        items: [
          {
            post_id: 52,
            topic_id: 5,
            author_username: 'mod',
            body: 'A reply',
            score: -3,
            mod_count: 2,
            reason: 'abusive',
            created_at: '2026-08-14T10:00:00Z',
          },
        ],
      }),
    );
    const { getModerationQueue } = await import('./forum');
    const res = await getModerationQueue();
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/moderation/queue');
    expect(res.count).toBe(1);
    expect(res.items[0].score).toBe(-3);
    expect(res.items[0].mod_count).toBe(2);
  });

  it('moderatePost POSTs /api/forum/posts/{id}/moderate with the reason', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, delta: -2, score_after: -2, hidden_until: null }));
    const { moderatePost } = await import('./forum');
    const res = await moderatePost(52, 'troll');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/posts/52/moderate');
    const init = mockFetch.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ reason: 'troll' });
    expect(res.delta).toBe(-2);
    expect(res.score_after).toBe(-2);
  });

  it('getPostModerations GETs /api/forum/posts/{id}/moderations', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        items: [{ moderator_id: 7, reason: 'insightful', delta: 2, score_after: 2, created_at: '2026-08-14T10:00:00Z' }],
      }),
    );
    const { getPostModerations } = await import('./forum');
    const res = await getPostModerations(52);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/posts/52/moderations');
    expect(res.items[0].reason).toBe('insightful');
  });

  it('hidePost POSTs /api/admin/forum/hide/{id}', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { hidePost } = await import('./forum');
    await hidePost(52);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/hide/52');
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('POST');
  });

  it('lockTopic and pinTopic POST the curator endpoints', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { lockTopic, pinTopic } = await import('./forum');
    await lockTopic(5);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/topics/5/lock');
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('POST');
    await pinTopic(5);
    expect(String(mockFetch.mock.calls[1][0])).toBe('/api/admin/forum/topics/5/pin');
  });

  it('createBan POSTs /api/admin/forum/bans with the ban body', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 9 }));
    const { createBan } = await import('./forum');
    const res = await createBan({ user_id: 2, scope: 'forum', reason: 'spam', expires_at: null });
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/bans');
    const init = mockFetch.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ user_id: 2, scope: 'forum', reason: 'spam', expires_at: null });
    expect(res.id).toBe(9);
  });

  it('listBans GETs /api/admin/forum/bans', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        items: [
          {
            id: 9,
            user_id: 2,
            user_username: 'spammer',
            category_slug: null,
            category_title: null,
            reason: 'spam',
            banned_by_username: 'curator',
            expires_at: null,
            created_at: '2026-08-14T10:00:00Z',
          },
        ],
      }),
    );
    const { listBans } = await import('./forum');
    const res = await listBans();
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/bans');
    expect(res.items[0].user_username).toBe('spammer');
  });

  it('liftBan DELETEs /api/admin/forum/bans/{id}', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { liftBan } = await import('./forum');
    await liftBan(9);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/bans/9');
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('DELETE');
  });

  it('reportForumPost POSTs /api/reports with forum_post target', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, report_id: 33 }));
    const { reportForumPost } = await import('./forum');
    const res = await reportForumPost(52, 'spam');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/reports');
    const init = mockFetch.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ target_type: 'forum_post', target_id: 52, reason: 'spam' });
    expect(res.report_id).toBe(33);
  });
});

describe('F6 metamoderation API client', () => {
  it('getMetamodQueue GETs /api/forum/metamod/queue and returns anonymized audit items', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        count: 2,
        items: [
          {
            action_id: 11,
            post_id: 52,
            topic_id: 5,
            excerpt: 'A reply that needed moderation',
            reason: 'troll',
            delta: -2,
            score_after: -2,
            created_at: '2026-08-14T10:00:00Z',
          },
          {
            action_id: 12,
            post_id: 53,
            topic_id: 5,
            excerpt: 'Actually a good point',
            reason: 'insightful',
            delta: 2,
            score_after: 4,
            created_at: '2026-08-14T11:00:00Z',
          },
        ],
      }),
    );
    const { getMetamodQueue } = await import('./forum');
    const res = await getMetamodQueue();
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/metamod/queue');
    expect(res.err).toBe(0);
    expect(res.count).toBe(2);
    expect(res.items).toHaveLength(2);
    // Moderator identity is anonymized — no moderator_id on items.
    expect(res.items[0]).not.toHaveProperty('moderator_id');
    expect(res.items[0]).toEqual({
      action_id: 11,
      post_id: 52,
      topic_id: 5,
      excerpt: 'A reply that needed moderation',
      reason: 'troll',
      delta: -2,
      score_after: -2,
      created_at: '2026-08-14T10:00:00Z',
    });
    expect(res.items[1].delta).toBe(2);
    expect(res.items[1].score_after).toBe(4);
  });

  it('surfaces pool_too_small when metamoderation is dormant', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, count: 0, items: [], pool_too_small: true }));
    const { getMetamodQueue } = await import('./forum');
    const res = await getMetamodQueue();
    expect(res.pool_too_small).toBe(true);
    expect(res.items).toEqual([]);
  });

  it('voteMetamod POSTs /api/forum/metamod/{actionId}/vote with the verdict', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { voteMetamod } = await import('./forum');
    const res = await voteMetamod(11, 'unfair');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/metamod/11/vote');
    const init = mockFetch.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe('POST');
    expect(JSON.parse(String(init.body))).toEqual({ verdict: 'unfair' });
    expect(res.err).toBe(0);
  });

  it('voteMetamod passes through the three valid verdicts', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { voteMetamod } = await import('./forum');
    for (const verdict of ['fair', 'unfair', 'unsure'] as const) {
      await voteMetamod(11, verdict);
      const init = mockFetch.mock.calls[mockFetch.mock.calls.length - 1][1] as RequestInit;
      expect(JSON.parse(String(init.body))).toEqual({ verdict });
    }
  });

  it('voteMetamod propagates non-zero err responses (409 already voted, 403 not eligible)', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 409, msg: 'already voted' }));
    const { voteMetamod } = await import('./forum');
    const res = await voteMetamod(11, 'fair');
    expect(res.err).toBe(409);
  });
});

describe('US2 pin/lock affordances', () => {
  it('pinTopic POSTs /api/admin/forum/topics/{id}/pin with pinned flag', async () => {
    const { pinTopic } = await import('./forum');
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 5, status: 'pinned' }));
    const res = await pinTopic(5, true);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/topics/5/pin');
    expect(JSON.parse(String((mockFetch.mock.calls[0][1] as RequestInit).body))).toEqual({ pinned: true });
    expect(res.status).toBe('pinned');
  });
  it('pinTopic toggles when no arg (empty object)', async () => {
    const { pinTopic } = await import('./forum');
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 5, status: 'open' }));
    await pinTopic(5);
    expect(JSON.parse(String((mockFetch.mock.calls[0][1] as RequestInit).body))).toEqual({});
  });
  it('lockTopic POSTs /api/admin/forum/topics/{id}/lock with locked flag', async () => {
    const { lockTopic } = await import('./forum');
    mockFetch.mockResolvedValue(okJson({ err: 0, id: 5, status: 'locked' }));
    const res = await lockTopic(5, true);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/admin/forum/topics/5/lock');
    expect(JSON.parse(String((mockFetch.mock.calls[0][1] as RequestInit).body))).toEqual({ locked: true });
    expect(res.status).toBe('locked');
  });
  it('locked post rejection surfaces topic_locked error', async () => {
    const { createPost } = await import('./forum');
    mockFetch.mockResolvedValue(okJson({ err: 403, msg: 'topic_locked' }));
    const res = await createPost(5, { body: 'hello' });
    expect(res.err).toBe(403);
    expect(res.msg).toContain('topic_locked');
  });
});

describe('US3 metamod queue polish', () => {
  it('getMetamodQueue with filter=unreviewed and pagination', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, items: [{ grant_id: 1, post_id: 2, reason: 'troll', created_at: '2026-08-14T10:00:00Z', my_verdict: null }], count: 1, next_cursor: 1 }));
    const { getMetamodQueue } = await import('./forum');
    const res = await getMetamodQueue({ filter: 'unreviewed', limit: 1 });
    expect(String(mockFetch.mock.calls[0][0])).toContain('filter=unreviewed');
    expect(res.next_cursor).toBe(1);
  });
  it('getMetamodQueue filter=reviewed verdict=fair', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, items: [], count: 0, next_cursor: null }));
    const { getMetamodQueue } = await import('./forum');
    await getMetamodQueue({ filter: 'reviewed', verdict: 'fair' });
    expect(String(mockFetch.mock.calls[0][0])).toContain('filter=reviewed');
    expect(String(mockFetch.mock.calls[0][0])).toContain('verdict=fair');
  });
  it('duplicate verdict 409 via grants endpoint', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 409, msg: 'already voted' }));
    const { voteMetamodGrant } = await import('./forum');
    const res = await voteMetamodGrant(1, 'fair');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/metamod/grants/1/verdict');
    expect(res.err).toBe(409);
  });
  it('getMetamodGrant returns anonymized context', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, grant_id: 1, post_id: 2, excerpt: 'body', reason: 'troll', delta: -2 }));
    const { getMetamodGrant } = await import('./forum');
    const res = await getMetamodGrant(1);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/forum/metamod/grants/1');
    expect((res as any).grant_id).toBe(1);
    expect(res).not.toHaveProperty('moderator_id');
  });
});
