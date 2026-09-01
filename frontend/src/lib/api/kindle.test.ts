import { describe, it, expect, vi, beforeEach } from 'vitest';

// Test the Send-to-Kindle client by mocking fetch + localStorage.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

async function importKindle() {
  return await import('./kindle');
}

describe('sendToKindle', () => {
  it('throws when the user has no token', async () => {
    const { sendToKindle } = await importKindle();
    await expect(sendToKindle({ url: 'https://ao3.org/works/1' })).rejects.toThrow(
      'You must be logged in to send to Kindle.',
    );
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('POSTs url with the Bearer token and parses the response', async () => {
    localStorage.setItem('fichub_token', 'kindle-jwt');
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, url_id: 'abc', to: 'me@kindle.com' }),
    });

    const { sendToKindle } = await importKindle();
    const res = await sendToKindle({ url: 'https://ao3.org/works/1' });

    expect(res.url_id).toBe('abc');
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/send-to-kindle');
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe('Bearer kindle-jwt');
    expect(JSON.parse(init.body).url).toBe('https://ao3.org/works/1');
  });

  it('POSTs url_id when no url is given', async () => {
    localStorage.setItem('fichub_token', 't');
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0 }) });
    const { sendToKindle } = await importKindle();
    await sendToKindle({ url_id: 'known-id' });
    expect(JSON.parse(mockFetch.mock.calls[0][1].body).url_id).toBe('known-id');
  });

  it('throws API error on non-OK response', async () => {
    localStorage.setItem('fichub_token', 't');
    mockFetch.mockResolvedValue({ ok: false, status: 401, text: async () => 'nope' });
    const { sendToKindle } = await importKindle();
    await expect(sendToKindle({ url: 'u' })).rejects.toThrow('API error 401');
  });
});
