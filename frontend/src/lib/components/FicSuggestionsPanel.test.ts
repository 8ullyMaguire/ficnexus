// Per-fic suggestions panel component tests: renders the suggestion list with
// scores + my_vote, the autocomplete/search form, the URL-paste mode, and the
// owner/admin remove link.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

let mockLoggedIn = false;
let mockRole = 0;
let mockUserId: number | null = null;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    get isLoggedIn() { return mockLoggedIn; },
    get user() {
      return mockLoggedIn ? { id: mockUserId ?? 1, username: 'u', role: mockRole, reputation: 0 } : null;
    },
    // F7: effective level — admin role (10) → 100 so the remove-link gate
    // (level ≥ 100) passes for the mock admin; everyone else stays at 0.
    get level() { return mockLoggedIn && mockRole >= 10 ? 100 : 0; },
    get exp() { return 0; },
    init: vi.fn(),
  },
}));

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function suggestionsPayload() {
  return jsonResponse({
    err: 0,
    url_id: 'fic-1',
    suggestions: [
      {
        id: 1,
        url_id: 'fic-1',
        suggested_url_id: 'fic-2',
        suggested_title: 'Companion Fic',
        suggested_author: 'Author B',
        comment: 'Read this after the anchor!',
        score: 3,
        total_votes: 5,
        my_vote: 1,
        user_id: 2,
      },
      {
        id: 2,
        url_id: 'fic-1',
        suggested_url_id: 'fic-3',
        suggested_title: 'Sister Fic',
        suggested_author: 'Author C',
        comment: null,
        score: -1,
        total_votes: 2,
        my_vote: 0,
        user_id: 3,
      },
    ],
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  mockLoggedIn = false;
  mockRole = 0;
  mockUserId = null;
  mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url.includes('/api/fic-suggestions?')) {
      return Promise.resolve(suggestionsPayload());
    }
    if (url.includes('/api/fic-suggestions') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, suggestion_id: 99 }));
    }
    if (url.includes('/api/fic-suggestions/') && init?.method === 'POST' && url.includes('/vote')) {
      return Promise.resolve(jsonResponse({ err: 0, new_score: 4, my_vote: 1 }));
    }
    if (url.includes('/api/fic-suggestions/') && init?.method === 'POST' && url.includes('/remove')) {
      return Promise.resolve(jsonResponse({ err: 0, removed: true }));
    }
    if (url.includes('/api/search?q=')) {
      return Promise.resolve(jsonResponse({
        err: 0,
        results: [
          { url_id: 'fic-9', title: 'Nine Lives', author: 'Author N' },
          { url_id: 'fic-10', title: 'Ten Tunes', author: 'Author T' },
        ],
      }));
    }
    return Promise.reject(new Error(`unexpected fetch: ${url}`));
  });
});

async function loadPanel() {
  return await import('./FicSuggestionsPanel.svelte');
}

describe('FicSuggestionsPanel list', () => {
  it('renders suggestions with title, author, comment and vote scores', async () => {
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });

    await waitFor(() => {
      expect(screen.getByText('Companion Fic')).toBeTruthy();
    });
    expect(screen.getByText(/by Author B/)).toBeTruthy();
    expect(screen.getByText('Read this after the anchor!')).toBeTruthy();
    expect(screen.getByText('Sister Fic')).toBeTruthy();
    // Link to the suggested fic's page
    expect(screen.getByRole('link', { name: /Companion Fic/ }).getAttribute('href')).toBe('/works/fic-2');
    // Score badges
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.getByText('-1')).toBeTruthy();
  });

  it('shows the empty state when there are no suggestions', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/fic-suggestions?')) {
        return Promise.resolve(jsonResponse({ err: 0, url_id: 'fic-1', suggestions: [] }));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });

    await waitFor(() => {
      expect(screen.getByText(/No suggestions yet/)).toBeTruthy();
    });
  });

  it('shows the remove link only for the owner or an admin', async () => {
    // Anonymous: no remove links (suggestions owned by users 2 and 3).
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });
    await waitFor(() => expect(screen.getByText('Companion Fic')).toBeTruthy());
    expect(screen.queryAllByText(/✕ remove/)).toHaveLength(0);
  });

  it('shows remove links for admin (level ≥ 100, derived from role 10)', async () => {
    mockLoggedIn = true;
    mockRole = 10;
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });
    await waitFor(() => expect(screen.getByText('Companion Fic')).toBeTruthy());
    expect(screen.getAllByText(/✕ remove/).length).toBeGreaterThanOrEqual(2);
  });
});

describe('FicSuggestionsPanel form', () => {
  it('suggests a fic via the in-DB autocomplete and refreshes the list', async () => {
    mockLoggedIn = true;
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });

    // Wait for the list, then type into the autocomplete.
    await waitFor(() => expect(screen.getByText('Companion Fic')).toBeTruthy());
    const input = screen.getByLabelText(/Search a fic to suggest/);
    await fireEvent.input(input, { target: { value: 'Nine' } });

    // Debounced autocomplete fires; pick the first result. The pickable
    // element is the button inside the li[role=option].
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Nine Lives/ })).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: /Nine Lives/ }));

    // Submit.
    const submitBtn = screen.getByRole('button', { name: /Suggest similar fic/ });
    await fireEvent.click(submitBtn);

    // POST went to /api/fic-suggestions with the picked url_id.
    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find(
        (c) => String(c[0]).includes('/api/fic-suggestions') && c[1]?.method === 'POST' && !String(c[0]).includes('/vote') && !String(c[0]).includes('/remove'),
      );
      expect(postCall).toBeTruthy();
      const body = JSON.parse(String(postCall![1].body));
      expect(body.suggested_url_id).toBe('fic-9');
    });
    // Success message + list reload.
    await waitFor(() => expect(screen.getByText('Suggestion added ✓')).toBeTruthy());
  });

  it('paste-a-link mode sends the url for scraping', async () => {
    mockLoggedIn = true;
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });

    await waitFor(() => expect(screen.getByText('Companion Fic')).toBeTruthy());
    await fireEvent.click(screen.getByRole('button', { name: /Paste a link/ }));

    const urlInput = screen.getByLabelText(/Fic URL to suggest/);
    await fireEvent.input(urlInput, { target: { value: 'https://archiveofourown.org/works/123' } });
    await fireEvent.click(screen.getByRole('button', { name: /Suggest similar fic/ }));

    // The submit posts immediately (no autocomplete debounce in URL mode).
    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find(
        (c) => String(c[0]).includes('/api/fic-suggestions') && c[1]?.method === 'POST' && !String(c[0]).includes('/vote') && !String(c[0]).includes('/remove'),
      );
      expect(postCall).toBeTruthy();
      const init = (postCall as unknown[])[1] as RequestInit;
      const body = JSON.parse(String(init.body));
      expect(body.url).toBe('https://archiveofourown.org/works/123');
    });
    // And the success message appears.
    await waitFor(() => expect(screen.getByText('Suggestion added ✓')).toBeTruthy());
  });

  it('blocks submission when logged out', async () => {
    mockLoggedIn = false;
    const { default: Panel } = await loadPanel();
    render(Panel, { props: { urlId: 'fic-1' } });

    await waitFor(() => expect(screen.getByText('Companion Fic')).toBeTruthy());
    await fireEvent.click(screen.getByRole('button', { name: /Suggest similar fic/ }));
    await waitFor(() => {
      expect(screen.getByText('Log in to suggest a fic.')).toBeTruthy();
    });
  });
});
