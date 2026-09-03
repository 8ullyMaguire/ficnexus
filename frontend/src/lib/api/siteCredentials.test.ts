import { describe, it, expect, vi, beforeEach } from 'vitest';

// Test the site-credentials API client (settings page) by mocking fetch.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function importClient() {
  return await import('./siteCredentials');
}

function okJson(body: unknown) {
  return { ok: true, status: 200, json: async () => body };
}

describe('listSiteCredentials', () => {
  it('GETs /api/user/site-credentials and returns the credential list', async () => {
    mockFetch.mockResolvedValue(
      okJson({
        err: 0,
        credentials: [
          { domain: 'fanfics.me', username: 'reader', expires_at: '2026-09-11T00:00:00Z' },
        ],
      }),
    );
    const { listSiteCredentials } = await importClient();
    const res = await listSiteCredentials();
    expect(mockFetch.mock.calls[0][0]).toBe('/api/user/site-credentials');
    expect(res.credentials).toHaveLength(1);
    expect(res.credentials[0].domain).toBe('fanfics.me');
    // The API contract never exposes a password.
    expect(JSON.stringify(res)).not.toContain('password');
  });
});

describe('setSiteCredentials', () => {
  it('PUTs domain/username/password to /api/user/site-credentials', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, domain: 'fanfics.me', username: 'reader', expires_at: '2026-09-11T00:00:00Z' }),
    );
    const { setSiteCredentials } = await importClient();
    const res = await setSiteCredentials('fanfics.me', 'reader', 'sup3r-secret');
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/user/site-credentials');
    expect(init.method).toBe('PUT');
    const body = JSON.parse(init.body);
    expect(body).toEqual({ domain: 'fanfics.me', username: 'reader', password: 'sup3r-secret' });
    expect(res.err).toBe(0);
  });
});

describe('deleteSiteCredentials', () => {
  it('DELETEs /api/user/site-credentials/{domain} (URL-encoded)', async () => {
    mockFetch.mockResolvedValue({ ok: true, status: 204, json: async () => ({}) });
    const { deleteSiteCredentials } = await importClient();
    await deleteSiteCredentials('inkbunny.net');
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/user/site-credentials/inkbunny.net');
    expect(init.method).toBe('DELETE');
  });
});

describe('api error handling', () => {
  it('throws on non-OK responses', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 401,
      text: async () => '{"err":401,"msg":"Login required"}',
    });
    const { listSiteCredentials } = await importClient();
    await expect(listSiteCredentials()).rejects.toThrow('API error 401');
  });
});
