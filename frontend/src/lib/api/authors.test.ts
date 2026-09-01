import { describe, it, expect, vi, beforeEach } from 'vitest';
import { buildAuthorSearchUrl } from './authors';

// Test the authors API client by mocking fetch
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

describe('buildAuthorSearchUrl', () => {
  it('omits q when empty', () => {
    expect(buildAuthorSearchUrl('')).toBe('/authors/search');
  });

  it('includes encoded q', () => {
    expect(buildAuthorSearchUrl('Harry Potter')).toBe('/authors/search?q=Harry+Potter');
  });
});

describe('authors API client', () => {
  it('searchAuthors hits GET /api/authors/search?q= and returns items', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        page: 1,
        items: [{ id: 1, canonical_name: 'Alice', bio: null, linked_authors: 2 }],
      }),
    });
    const { searchAuthors } = await import('./authors');
    const res = await searchAuthors('Alice');
    expect(mockFetch.mock.calls[0][0]).toBe('/api/authors/search?q=Alice');
    expect(res).toHaveLength(1);
    expect(res[0].canonical_name).toBe('Alice');
  });

  it('getAuthor merges nested profile/socials/linked_authors', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        profile: { id: 7, canonical_name: 'Bob', bio: 'hi', avatar_url: null, created_at: '2026-01-01T00:00:00' },
        socials: [{ id: 1, platform: 'ao3', url: 'https://archiveofourown.org/users/Bob', label: null, is_visible: true }],
        linked_authors: [],
      }),
    });
    const { getAuthor } = await import('./authors');
    const res = await getAuthor(7);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/authors/7');
    expect(res.id).toBe(7);
    expect(res.socials).toHaveLength(1);
    expect(res.linked_authors).toEqual([]);
  });

  it('updateAuthor sends PUT with bio/avatar_url', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, msg: 'Profile updated' }) });
    const { updateAuthor } = await import('./authors');
    await updateAuthor(3, { bio: 'new bio' });
    expect(mockFetch.mock.calls[0][0]).toBe('/api/authors/3');
    expect(mockFetch.mock.calls[0][1].method).toBe('PUT');
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.bio).toBe('new bio');
  });

  it('addSocial sends POST with platform/url/label', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, msg: 'Social link added' }) });
    const { addSocial } = await import('./authors');
    await addSocial(3, { platform: 'xitter', url: 'https://x.com/bob', label: 'X' });
    expect(mockFetch.mock.calls[0][0]).toBe('/api/authors/3/socials');
    expect(mockFetch.mock.calls[0][1].method).toBe('POST');
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.platform).toBe('xitter');
    expect(body.url).toBe('https://x.com/bob');
  });

  it('removeSocial sends DELETE to /api/authors/{id}/socials/{socialId}', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, msg: 'Social link removed' }) });
    const { removeSocial } = await import('./authors');
    await removeSocial(3, 9);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/authors/3/socials/9');
    expect(mockFetch.mock.calls[0][1].method).toBe('DELETE');
  });

  it('proposeMerge sends POST with source_author/source_url/target_profile_id', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, msg: 'Merge proposal submitted' }) });
    const { proposeMerge } = await import('./authors');
    await proposeMerge('Alice', 'https://archiveofourown.org/users/Alice', 1);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/curator/authors/merge');
    expect(mockFetch.mock.calls[0][1].method).toBe('POST');
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.source_author).toBe('Alice');
    expect(body.source_url).toBe('https://archiveofourown.org/users/Alice');
    expect(body.target_profile_id).toBe(1);
  });

  it('getPendingMerges returns items', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        items: [
          { id: 1, source_author: 'Alice', source_url: 'https://x.com/a', target_profile_id: 1, target_name: 'Alice', proposed_by: 'curator1', created_at: '2026-01-01' },
        ],
      }),
    });
    const { getPendingMerges } = await import('./authors');
    const res = await getPendingMerges();
    expect(mockFetch.mock.calls[0][0]).toBe('/api/curator/authors/pending');
    expect(res).toHaveLength(1);
    expect(res[0].target_name).toBe('Alice');
  });

  it('approveMerge sends POST to /api/curator/authors/approve/{id}', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, msg: 'Merge approved' }) });
    const { approveMerge } = await import('./authors');
    await approveMerge(5);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/curator/authors/approve/5');
    expect(mockFetch.mock.calls[0][1].method).toBe('POST');
  });

  it('rejectMerge sends POST to /api/curator/authors/reject/{id}', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, msg: 'Merge rejected' }) });
    const { rejectMerge } = await import('./authors');
    await rejectMerge(6);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/curator/authors/reject/6');
    expect(mockFetch.mock.calls[0][1].method).toBe('POST');
  });

  it('throws on non-ok response', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 403,
      text: async () => 'Curator access required',
    });
    const { getPendingMerges } = await import('./authors');
    await expect(getPendingMerges()).rejects.toThrow('API error 403');
  });
});
