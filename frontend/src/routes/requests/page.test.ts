import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  // Default to modern UI in tests (archive is the app default)
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

function requestPayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    items: [
      { id: 1, title: 'Slow burn recs', body: '', seed_work_id: null, status: 'open', created_at: '2026-08-08', answer_count: 3 },
      { id: 2, title: 'Dark Harry no bashing', body: '', seed_work_id: null, status: 'open', created_at: '2026-08-07', answer_count: 0 },
    ],
    page: 1,
    status: 'open',
    ...overrides,
  };
}

describe('requests list page', () => {
  it('renders open requests from the API', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/requests')) {
        return Promise.resolve(jsonResponse(requestPayload()));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Slow burn recs')).toBeTruthy();
    });
    expect(screen.getByText('Dark Harry no bashing')).toBeTruthy();
    expect(screen.getByText(/3 answers/)).toBeTruthy();
    expect(screen.getByText('Post New Request')).toBeTruthy();
  });
});

describe('requests new page', () => {
  it('posts the request with honeypot form_opened_at and redirects', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url === '/api/requests' && init?.method === 'POST') {
        const body = JSON.parse(String(init.body));
        return Promise.resolve(jsonResponse({ err: 0, id: 99 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./new/+page.svelte');
    render(Page);

    await fireEvent.input(screen.getByPlaceholderText(/e.g. Fics like/), { target: { value: 'Fics like The Awakening' } });
    await fireEvent.click(screen.getByRole('button', { name: /Post request/i }));

    await waitFor(() => {
      const call = mockFetch.mock.calls.find((c) => String(c[0]) === '/api/requests');
      expect(call).toBeTruthy();
      const body = JSON.parse(String(call![1]!.body));
      expect(body.title).toBe('Fics like The Awakening');
    });
  });
});
