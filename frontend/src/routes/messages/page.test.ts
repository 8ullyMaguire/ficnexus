import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    level: 10,
    exp: 1000,
    user: { id: 1, username: 'alice', role: 1, reputation: 0, level: 10, email: null, locale: 'en' },
  },
}));

vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ url: new URL('http://localhost/messages') });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

function mockRoomsAndThread() {
  mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url.includes('/api/messages/rooms?filter=')) {
      return Promise.resolve(
        jsonResponse({
          err: 0,
          rooms: [{ id: 9, name: null, is_group: false, created_at: '2026-09-07T10:00:00Z', last_activity_at: null }],
          count: 1,
        }),
      );
    }
    if (url.includes('/api/messages/rooms/9/messages') && (!init?.method || init.method === 'GET')) {
      return Promise.resolve(
        jsonResponse({
          err: 0,
          messages: [
            { id: 3, sender_id: 2, sender_username: 'bob', body: 'hey alice', created_at: '2026-09-07T10:01:00Z' },
          ],
          has_more: false,
        }),
      );
    }
    return Promise.reject(new Error(`unexpected: ${url}`));
  });
}

describe('messages page', () => {
  it('lists rooms and renders the thread', async () => {
    mockRoomsAndThread();

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('hey alice')).toBeTruthy();
    });
    expect(screen.getByText('bob')).toBeTruthy();
  });

  it('sends a message via POST and clears the composer', async () => {
    mockRoomsAndThread();
    // After the initial thread load, the send POST lands here.
    const origImpl = mockFetch.getMockImplementation();
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes('/api/messages/rooms/9/messages') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, message_id: 4 }));
      }
      return origImpl!(input, init);
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('hey alice')).toBeTruthy();
    });
    const box = screen.getByPlaceholderText('Write a message…');
    await fireEvent.input(box, { target: { value: 'hello bob' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Send' }));

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/messages/rooms/9/messages'),
        expect.objectContaining({ method: 'POST' }),
      );
    });
    await waitFor(() => {
      expect((screen.getByPlaceholderText('Write a message…') as HTMLTextAreaElement).value).toBe('');
    });
  });
});
