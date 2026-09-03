import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';

// Extend coverage of the v2 social client: reading lists, shelves, follows,
// notifications, badges, trending, proposals, and error paths.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

async function importSocial() {
  return await import('./social');
}

describe('batch download client', () => {
  // jsdom has no URL.createObjectURL — stub it (and revoke) so the helpers'
  // blob-download path runs end to end.
  const createObjectURL = vi.fn(() => 'blob:mock-url');
  const revokeObjectURL = vi.fn();
  beforeAll(() => {
    URL.createObjectURL = createObjectURL as unknown as typeof URL.createObjectURL;
    URL.revokeObjectURL = revokeObjectURL as unknown as typeof URL.revokeObjectURL;
  });

  it('downloadAuthorWorks GETs /api/download/author with the encoded author URL', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      blob: async () => new Blob(['zip'], { type: 'application/zip' }),
      headers: new Headers({ 'content-disposition': 'attachment; filename="author_works.zip"' }),
    });
    const { downloadAuthorWorks } = await importSocial();
    await downloadAuthorWorks('https://archiveofourown.org/users/TestAuthor');
    const [url, init] = mockFetch.mock.calls[0];
    expect(String(url)).toContain('/api/download/author?url=');
    expect(String(url)).toContain(encodeURIComponent('https://archiveofourown.org/users/TestAuthor'));
    expect(init.method ?? 'GET').toBe('GET');
  });

  it('downloadSeries GETs /api/download/series with the encoded series URL', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      blob: async () => new Blob(['zip'], { type: 'application/zip' }),
      headers: new Headers({ 'content-disposition': 'attachment; filename="series_42.zip"' }),
    });
    const { downloadSeries } = await importSocial();
    await downloadSeries('https://archiveofourown.org/series/42');
    const [url, init] = mockFetch.mock.calls[0];
    expect(String(url)).toContain('/api/download/series?url=');
    expect(String(url)).toContain(encodeURIComponent('https://archiveofourown.org/series/42'));
    expect(init.method ?? 'GET').toBe('GET');
  });

  it('downloadSeries throws with the HTTP status on a failed response', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 404,
      text: async () => 'no works found for this series',
    });
    const { downloadSeries } = await importSocial();
    await expect(downloadSeries('https://archiveofourown.org/series/99')).rejects.toThrow(
      'Download failed (404)',
    );
  });
});


describe('reading lists', () => {
  it('createReadingList POSTs title/description/is_public', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, list: { id: 1 } }));
    const { createReadingList } = await importSocial();
    await createReadingList('My List', 'desc', true);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/lists');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toEqual({ title: 'My List', description: 'desc', is_public: true });
  });

  it('getReadingListDetail GETs /api/lists/{id}', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, list: { id: 1 }, items: [] }));
    const { getReadingListDetail } = await importSocial();
    const res = await getReadingListDetail(1);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/lists/1');
    expect(res.list.id).toBe(1);
  });

  it('updateReadingList PATCHes the patch object', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { updateReadingList } = await importSocial();
    await updateReadingList(1, { title: 'Renamed' });
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/lists/1');
    expect(init.method).toBe('PATCH');
    expect(JSON.parse(init.body).title).toBe('Renamed');
  });

  it('addReadingListItem POSTs work_id + blurb', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, position: 1 }));
    const { addReadingListItem } = await importSocial();
    const res = await addReadingListItem(1, 42, 'great');
    expect(res.position).toBe(1);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.work_id).toBe(42);
    expect(body.blurb).toBe('great');
  });

  it('removeReadingListItem DELETEs /api/lists/{id}/items/{work_id}', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, removed: true }));
    const { removeReadingListItem } = await importSocial();
    await removeReadingListItem(1, 42);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/lists/1/items/42');
    expect(mockFetch.mock.calls[0][1].method).toBe('DELETE');
  });
});

describe('shelves', () => {
  it('listWorksInShelf GETs /api/shelves/{id}/works', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, works: [] }));
    const { listWorksInShelf } = await importSocial();
    await listWorksInShelf(3);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/shelves/3/works');
  });

  it('addWorkToShelf POSTs shelf_id + work_id', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { addWorkToShelf } = await importSocial();
    await addWorkToShelf(3, 42);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body).toEqual({ shelf_id: 3, work_id: 42 });
  });
});

describe('reading status', () => {
  it('updateReadingStatus POSTs status + current_chapter', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { updateReadingStatus } = await importSocial();
    await updateReadingStatus(42, 'reading', 3);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.status).toBe('reading');
    expect(body.current_chapter).toBe(3);
  });

  it('getReadingList omits the status query when not given', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, reading_list: [] }));
    const { getReadingList } = await importSocial();
    await getReadingList();
    expect(mockFetch.mock.calls[0][0]).toBe('/api/reading/list');
  });

  it('getReadingList appends ?status= when given', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, reading_list: [] }));
    const { getReadingList } = await importSocial();
    await getReadingList('completed');
    expect(mockFetch.mock.calls[0][0]).toBe('/api/reading/list?status=completed');
  });
});

describe('follows & updates', () => {
  it('follow POSTs target_type/target_id/author_name', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { follow } = await importSocial();
    await follow('work', 42);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.target_type).toBe('work');
    expect(body.target_id).toBe(42);
  });

  it('checkWorkFollow GETs /api/follows/check/work/{id}', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, follow_id: 9, is_following: true }));
    const { checkWorkFollow } = await importSocial();
    const res = await checkWorkFollow(42);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/follows/check/work/42');
    expect(res.is_following).toBe(true);
  });

  it('markAllUpdatesSeen settles all markFollowSeen calls', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, updated: true }));
    const { markAllUpdatesSeen } = await importSocial();
    await markAllUpdatesSeen([{ follow_id: 1 }, { follow_id: 2 }]);
    expect(mockFetch.mock.calls.length).toBe(2);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/v1/follows/1/seen');
  });

  it('refreshFic POSTs /api/v1/works/{url_id}/refresh', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, status: 'ok' }));
    const { refreshFic } = await importSocial();
    const res = await refreshFic('fic id');
    expect(mockFetch.mock.calls[0][0]).toBe('/api/v1/works/fic%20id/refresh');
    expect(res.status).toBe('ok');
  });
});

describe('notifications & badges', () => {
  it('listNotifications GETs with limit/offset', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, notifications: [], unread_count: 0 }));
    const { listNotifications } = await importSocial();
    await listNotifications(5, 10);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/notifications?limit=5&offset=10');
  });

  it('markNotificationRead POSTs the read endpoint', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { markNotificationRead } = await importSocial();
    await markNotificationRead(3);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/notifications/3/read');
    expect(mockFetch.mock.calls[0][1].method).toBe('POST');
  });

  it('updateNotificationPrefs PUTs the prefs', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0 }));
    const { updateNotificationPrefs } = await importSocial();
    await updateNotificationPrefs({ comment_reply: false });
    expect(mockFetch.mock.calls[0][1].method).toBe('PUT');
    expect(JSON.parse(mockFetch.mock.calls[0][1].body)).toEqual({ comment_reply: false });
  });

  it('listBadgeDefinitions GETs /api/badges', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, badges: [] }));
    const { listBadgeDefinitions } = await importSocial();
    await listBadgeDefinitions();
    expect(mockFetch.mock.calls[0][0]).toBe('/api/badges');
  });

  it('getUserBadges GETs /api/users/{id}/badges', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, badges: [] }));
    const { getUserBadges } = await importSocial();
    await getUserBadges(7);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/users/7/badges');
  });
});

describe('trending & quests & stats', () => {
  it('getTrending GETs /api/trending?days&limit', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, trending: [] }));
    const { getTrending } = await importSocial();
    await getTrending(14, 5);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/trending?days=14&limit=5');
  });

  it('getTrendingByTag encodes the tag name', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, tag: { id: 1, name: 'x', tag_type_id: 4 }, trending: [] }));
    const { getTrendingByTag } = await importSocial();
    await getTrendingByTag(4, 'Dark Harry');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/trending/tag/4/Dark%20Harry?days=7&limit=20');
  });

  it('getReadingStats GETs /api/users/{id}/reading-stats', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, total_words_read: 10, total_works_read: 1, login_streak: 1, recent: [] }));
    const { getReadingStats } = await importSocial();
    const res = await getReadingStats(7);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/users/7/reading-stats');
    expect(res.total_words_read).toBe(10);
  });
});

describe('proposals', () => {
  it('createProposal POSTs action_type + details', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, proposal_id: 5 }));
    const { createProposal } = await importSocial();
    const res = await createProposal('merge', 1, 2, undefined, { why: 'x' });
    expect(res.proposal_id).toBe(5);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.action_type).toBe('merge');
    expect(body.details).toEqual({ why: 'x' });
  });

  it('voteProposal POSTs the vote', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, vote_sum: 2, voter_count: 1 }));
    const { voteProposal } = await importSocial();
    const res = await voteProposal(5, 1);
    expect(res.vote_sum).toBe(2);
    expect(JSON.parse(mockFetch.mock.calls[0][1].body).vote).toBe(1);
  });
});

describe('generic request error path', () => {
  it('throws API error with status + body text', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 500, text: async () => 'boom' });
    const { getLeaderboard } = await importSocial();
    await expect(getLeaderboard()).rejects.toThrow('API error 500: boom');
  });
});
