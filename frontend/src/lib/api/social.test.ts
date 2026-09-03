import { describe, it, expect, vi, beforeEach } from 'vitest';
import { buildSearchQuery } from './search';

// Test the v2 client by mocking fetch
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

// Mock location so the 401 redirect path in request() doesn't blow up
// jsdom's "not implemented: navigation" stub during the 401 test.
globalThis.location = {
  pathname: '/',
  href: 'http://localhost/',
} as unknown as Location;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

describe('v2 auth client', () => {
  it('stores token on successful login', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        token: 'test-jwt-token',
        user: { id: 1, username: 'alice', role: 0, reputation: 0 },
      }),
    });
    const { login } = await import('./social');
    const res = await login('alice', 'pass123');
    expect(res.err).toBe(0);
    expect(localStorage.getItem('fichub_token')).toBe('test-jwt-token');
  });

  it('stores token on successful register', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        token: 'reg-token',
        user: { id: 2, username: 'bob', role: 0, reputation: 0 },
      }),
    });
    const { register } = await import('./social');
    await register('bob', 'pass456', 'bob@example.com');
    expect(localStorage.getItem('fichub_token')).toBe('reg-token');
  });

  it('sends honeypot form_opened_at + empty website on register', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, token: 'reg-token', user: { id: 3, username: 'carl', role: 0, reputation: 0 } }),
    });
    const { register } = await import('./social');
    await register('carl', 'pass789', 'carl@example.com', 'INVITE');
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.website).toBe('');
    expect(typeof body.form_opened_at).toBe('string');
    const opened = Number(body.form_opened_at);
    expect(Number.isFinite(opened)).toBe(true);
    // Must be a recent epoch-ms timestamp (within a minute of now).
    expect(Math.abs(Date.now() - opened)).toBeLessThan(60_000);
    expect(body.invite_code).toBe('INVITE');
  });

  it('sends auth header when token is stored', async () => {
    localStorage.setItem('fichub_token', 'my-token');
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, bookmarks: [] }),
    });
    const { listBookmarks } = await import('./social');
    await listBookmarks();
    const headers = mockFetch.mock.calls[0][1].headers;
    expect(headers.Authorization).toBe('Bearer my-token');
  });
});

describe('v2 bookmarks client', () => {
  it('sends POST with work_id', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0 }),
    });
    const { addBookmark } = await import('./social');
    await addBookmark(42, 'great fic', false);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.work_id).toBe(42);
    expect(body.notes).toBe('great fic');
  });
});

describe('v2 ratings client (5-star scale)', () => {
  it('sends a 1..=5 rating value and returns the positive-only aggregate', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        avg_rating: 4.5,
        rating_count: 2,
        likes: 3,
        review_count: 1,
        rating_distribution: { '4': 1, '5': 1 },
      }),
    });
    const { rateWork } = await import('./social');
    const res = await rateWork(7, 5);
    expect(res.avg_rating).toBe(4.5);
    expect(res.review_count).toBe(1);
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.rating).toBe(5);
    expect(body.work_id).toBe(7);
    // The public aggregate never carries dislikes.
    expect('dislikes' in res).toBe(false);
  });
});

describe('v2 kudos client (AO3-style anonymous-appreciable likes)', () => {
  it('POSTs /api/kudos/{work_id} to give a kudo and returns the new count', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, work_id: 7, kudos_count: 5, my_kudos: true }),
    });
    const { giveKudos } = await import('./social');
    const res = await giveKudos(7);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/kudos/7');
    expect(init.method).toBe('POST');
    expect(res.err).toBe(0);
    expect(res.kudos_count).toBe(5);
    expect(res.my_kudos).toBe(true);
  });

  it('DELETEs /api/kudos/{work_id} to remove a kudo', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, work_id: 7, kudos_count: 4, my_kudos: false }),
    });
    const { removeKudos } = await import('./social');
    const res = await removeKudos(7);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/kudos/7');
    expect(init.method).toBe('DELETE');
    expect(res.kudos_count).toBe(4);
    expect(res.my_kudos).toBe(false);
  });

  it('GETs /api/kudos/{work_id} publicly with the guest count and my_kudos flag', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, work_id: 7, kudos_count: 5, guest_count: 1, my_kudos: false }),
    });
    const { getKudos } = await import('./social');
    const res = await getKudos(7);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/kudos/7');
    expect(init.method ?? 'GET').toBe('GET');
    expect(res.kudos_count).toBe(5);
    expect(res.guest_count).toBe(1);
    expect(res.my_kudos).toBe(false);
  });

  it('surfaces the logged-in user kudo state from GET', async () => {
    localStorage.setItem('fichub_token', 'kudos-token');
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, work_id: 7, kudos_count: 3, my_kudos: true }),
    });
    const { getKudos } = await import('./social');
    const res = await getKudos(7);
    const headers = mockFetch.mock.calls[0][1].headers;
    expect(headers.Authorization).toBe('Bearer kudos-token');
    expect(res.my_kudos).toBe(true);
  });

  it('throws an API error when the kudos endpoint rejects (e.g. not logged in)', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      text: async () => '{"err":401,"msg":"Login required"}',
    });
    const { giveKudos } = await import('./social');
    await expect(giveKudos(7)).rejects.toThrow('API error 401');
  });
});

describe('v2 reviews client', () => {
  it('POSTs a review with work_id, rating, title and body', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        review: { id: 42, work_id: 7, rating: 5, title: 'Loved it', body: 'Great pacing', constructive: true, created_at: '2024-01-01T00:00:00Z', updated_at: '2024-01-01T00:00:00Z' },
      }),
    });
    const { postReview } = await import('./social');
    await postReview(7, 5, 'Loved it', 'Great pacing');
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/reviews');
    const body = JSON.parse(init.body);
    expect(body.work_id).toBe(7);
    expect(body.rating).toBe(5);
    expect(body.title).toBe('Loved it');
    expect(body.body).toBe('Great pacing');
  });

  it('GETs the review list for a work', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, work_id: 7, total: 1, reviews: [] }),
    });
    const { listReviews } = await import('./social');
    const res = await listReviews(7);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/works/7/reviews');
    expect(res.total).toBe(1);
  });

  it('DELETEs a review by id', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, msg: 'Review deleted' }),
    });
    const { deleteReview } = await import('./social');
    await deleteReview(42);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/reviews/42');
    expect(init.method).toBe('DELETE');
  });
});

describe('v2 comments client', () => {
  it('sends comment body', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, comment_id: 42 }),
    });
    const { addComment } = await import('./social');
    await addComment(7, 'Great story!');
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.body).toBe('Great story!');
    expect(body.work_id).toBe(7);
  });

  it('passes honeypot/timing fields through when provided', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, comment_id: 43 }),
    });
    const { addComment } = await import('./social');
    await addComment(7, 'Nice work.', undefined, {
      form_opened_at: '1700000000000',
      website: '',
    });
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.form_opened_at).toBe('1700000000000');
    expect(body.website).toBe('');
  });
});

describe('v2 user data export client', () => {
  it('GETs /api/user/export with the auth header and returns the response', async () => {
    localStorage.setItem('fichub_token', 'export-token');
    mockFetch.mockResolvedValue({
      ok: true,
      blob: async () => new Blob(['zip-bytes'], { type: 'application/zip' }),
    });
    const { exportUserData } = await import('./social');
    const res = await exportUserData();
    expect(res).toBeTruthy();
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/user/export');
    expect(init.method ?? 'GET').toBe('GET');
    expect(init.headers.Authorization).toBe('Bearer export-token');
  });

  it('throws on non-OK response', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 400,
      text: async () => '{"err":401,"msg":"Login required"}',
    });
    const { exportUserData } = await import('./social');
    await expect(exportUserData()).rejects.toThrow('Export failed (400)');
  });

  it('omits the Authorization header when no token is stored', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      blob: async () => new Blob(['zip'], { type: 'application/zip' }),
    });
    const { exportUserData } = await import('./social');
    await exportUserData();
    const [, init] = mockFetch.mock.calls[0];
    expect(init.headers.Authorization).toBeUndefined();
  });
});
