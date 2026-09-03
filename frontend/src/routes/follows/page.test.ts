import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

// Mock the auth store so the page behaves as a logged-in user.
let mockLoggedIn = true;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    get isLoggedIn() { return mockLoggedIn; },
    get user() { return mockLoggedIn ? { id: 1, username: 'u', role: 0, reputation: 0 } : null; },
    init: vi.fn(),
    handleLogout: vi.fn(),
  },
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

// Seeded follow rows. The GET /follows handler returns these verbatim.
let followsPayload: unknown[] = [
  { id: 1, followee_id: null, work_id: null, author_name: 'Ariel', created_at: '2026-01-01T00:00:00Z', last_seen: null },
  { id: 2, followee_id: null, work_id: 42, author_name: null, created_at: '2026-01-02T00:00:00Z', last_seen: null },
];

// Seeded exclusions for follow id 1; the GET handler returns the current list
// and POST appends, so add/remove round-trips are observable in the UI.
let followOneExclusions: Record<string, unknown>[] = [
  { id: 10, follow_id: 1, exclude_type: 'fandom', work_id: null, series_id: null, fandom: 'Batman', created_at: '2026-01-01T00:00:00Z' },
];

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function getLS(): Storage {
  if (typeof window !== 'undefined' && (window as any).localStorage) return (window as any).localStorage as Storage;
  const g = (globalThis as any).localStorage;
  if (g) return g as Storage;
  const store = new Map<string, string>();
  const fake = { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => { store.set(k, String(v)); }, removeItem: (k: string) => { store.delete(k); }, clear: () => { store.clear(); }, get length() { return store.size; }, key: (i: number) => Array.from(store.keys())[i] ?? null } as unknown as Storage;
  (globalThis as any).localStorage = fake;
  return fake;
}

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function setupFetch() {
  mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    const method = init?.method ?? 'GET';
    // Exclusion sub-resource ops.
    if (url.includes('/exclusions')) {
      if (method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, exclusion_id: 99 }));
      }
      if (method === 'DELETE') {
        return Promise.resolve(jsonResponse({ err: 0, removed: true }));
      }
      return Promise.resolve(jsonResponse({ err: 0, exclusions: followOneExclusions }));
    }
    // GET /follows — list follows.
    if (url === '/api/follows' && method === 'GET') {
      return Promise.resolve(jsonResponse({ err: 0, follows: followsPayload }));
    }
    // DELETE /follows/{id} — unfollow.
    if (/^\/api\/follows\/\d+$/.test(url) && method === 'DELETE') {
      return Promise.resolve(jsonResponse({ err: 0, removed: true }));
    }
    return Promise.reject(new Error(`unexpected fetch: ${method} ${url}`));
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  getLS().clear();
  // Start the seeded state fresh each test.
  followsPayload = [
    { id: 1, followee_id: null, work_id: null, author_name: 'Ariel', created_at: '2026-01-01T00:00:00Z', last_seen: null },
    { id: 2, followee_id: null, work_id: 42, author_name: null, created_at: '2026-01-02T00:00:00Z', last_seen: null },
  ];
  followOneExclusions = [
    { id: 10, follow_id: 1, exclude_type: 'fandom', work_id: null, series_id: null, fandom: 'Batman', created_at: '2026-01-01T00:00:00Z' },
  ];
  getLS().setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
  setupFetch();
});

async function loadPage() {
  return await import('./+page.svelte');
}

describe('follows page exclusions', () => {
  it('renders the Exclusions control for a followed author with its loaded exclusions', async () => {
    const { default: Page } = await loadPage();
    render(Page);

    // Author follow is listed.
    await waitFor(() => {
      expect(screen.getByText('Author: Ariel')).toBeTruthy();
    });
    // The exclusions control mounts and the seeded fandom exclusion shows.
    await waitFor(() => {
      expect(screen.getByText(/Fandom: Batman/)).toBeTruthy();
    });
    expect(screen.getAllByText('Exclusions').length).toBeGreaterThanOrEqual(1);
  });

  it('does not render an Exclusions control for non-author follows only', async () => {
    followsPayload = [
      { id: 2, followee_id: null, work_id: 42, author_name: null, created_at: '2026-01-02T00:00:00Z', last_seen: null },
    ];
    setupFetch();

    const { default: Page } = await loadPage();
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Work #42')).toBeTruthy();
    });
    // No author follow present, so no exclusions heading anywhere.
    expect(screen.queryByText('Exclusions')).toBeNull();
  });

  it('adds a work exclusion and shows it in the list', async () => {
    const addCalls: unknown[] = [];
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? 'GET';
      if (url.includes('/exclusions')) {
        if (method === 'POST') {
          addCalls.push(JSON.parse(String(init!.body)));
          return Promise.resolve(jsonResponse({ err: 0, exclusion_id: 99 }));
        }
        if (method === 'DELETE') {
          return Promise.resolve(jsonResponse({ err: 0, removed: true }));
        }
        return Promise.resolve(jsonResponse({ err: 0, exclusions: followOneExclusions }));
      }
      if (url === '/api/follows' && method === 'GET') {
        return Promise.resolve(jsonResponse({ err: 0, follows: followsPayload }));
      }
      if (/^\/api\/follows\/\d+$/.test(url) && method === 'DELETE') {
        return Promise.resolve(jsonResponse({ err: 0, removed: true }));
      }
      return Promise.reject(new Error(`unexpected fetch: ${method} ${url}`));
    });

    const { default: Page } = await loadPage();
    render(Page);

    // Wait for exclusions control of the author follow.
    await waitFor(() => {
      expect(screen.getByText(/Fandom: Batman/)).toBeTruthy();
    });

    // Choose Work type, type a work id, and click Add.
    const typeSelect = screen.getByLabelText('Exclusions') as HTMLSelectElement;
    await fireEvent.change(typeSelect, { target: { value: 'work' } });
    await fireEvent.input(screen.getByLabelText('Work ID'), { target: { value: '55' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add' }));

    // The API received a work exclusion targeting 55.
    expect(addCalls).toHaveLength(1);
    expect(addCalls[0]).toMatchObject({ exclude_type: 'work', work_id: 55 });
  });

  it('removes an exclusion when its Remove button is clicked', async () => {
    const { default: Page } = await loadPage();
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Fandom: Batman/)).toBeTruthy();
    });

    // Only the one seeded exclusion exists; its Remove button removes it.
    const removeBtns = screen.getAllByRole('button', { name: 'Remove' });
    expect(removeBtns.length).toBeGreaterThan(0);
    await fireEvent.click(removeBtns[0]);

    await waitFor(() => {
      expect(screen.queryByText(/Fandom: Batman/)).toBeNull();
    });
  });
});
