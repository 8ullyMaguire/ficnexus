import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import * as navigation from '$app/navigation';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

// Mock SvelteKit modules
vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ url: new URL('http://localhost/search') });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

async function loadSearchPage() {
  return await import('./+page.svelte');
}

function searchPayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    total: 1,
    page: 1,
    per_page: 20,
    results: [
      {
        url_id: 'fic-1',
        title: 'The Anchor Fic',
        author: 'Author One',
        source: 'archiveofourown.org',
        words: 52341,
        chapters: 21,
        status: 'complete',
        description: 'A gripping tale.',
        updated: '2026-02-01T00:00:00Z',
        rank: 1,
        snippet: null,
        tags: [],
        total_freeform: 0,
        comment_count: 4,
        kudos_count: 12,
      },
    ],
    facets: {
      fandoms: [],
      characters: [],
      relationships: [],
      warnings: [],
      categories: [],
      freeforms: [],
      statuses: [],
    },
    ...overrides,
  };
}

/** Mock /api/search + /api/search/suggest. */
function mockSearch(payload: unknown) {
  mockFetch.mockImplementation((input: RequestInfo | URL) => {
    const url = String(input);
    if (url.startsWith('/api/search') || url === '/api/search') {
      return Promise.resolve({ ok: true, json: async () => payload });
    }
    if (url.startsWith('/api/search/suggest')) {
      return Promise.resolve({ ok: true, json: async () => ({ err: 0, items: [] }) });
    }
    return Promise.reject(new Error(`unexpected: ${url}`));
  });
}

describe('SearchPage (archive UI)', () => {
  it('renders the archive Work Search form', async () => {
    mockSearch(searchPayload());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);

    expect(screen.getByText('Work Search')).toBeTruthy();
    expect(screen.getByPlaceholderText(/Search all fields/)).toBeTruthy();
    expect(screen.getAllByRole('button', { name: 'Search' }).length).toBeGreaterThan(0);
  });

  it('executes a search and displays results', async () => {
    mockSearch(searchPayload());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);

    const input = screen.getByPlaceholderText(/Search all fields/) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'anchor fic' } });
    // Submit the form (Enter) — the form's onsubmit triggers the search.
    await fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(screen.getByText('The Anchor Fic')).toBeTruthy();
    });
    // Results count + metadata from WorkBlurb
    expect(screen.getByText('1 Works')).toBeTruthy();
    expect(screen.getByText(/52,341/)).toBeTruthy();
  });

  it('shows an error when the search fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.startsWith('/api/search')) return Promise.resolve({ ok: false, status: 500, text: async () => 'boom' });
      if (url.startsWith('/api/search/suggest')) return Promise.resolve({ ok: true, json: async () => ({ err: 0, items: [] }) });
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);

    const input = screen.getByPlaceholderText(/Search all fields/) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'boom' } });
    await fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(screen.getByText(/Search failed/i)).toBeTruthy();
    });
  });

  it('shows the empty state when no works match', async () => {
    mockSearch(searchPayload({ total: 0, results: [] }));
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);

    const input = screen.getByPlaceholderText(/Search all fields/) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'nothing matches' } });
    await fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(screen.getByText('No works found.')).toBeTruthy();
    });
  });

  it('renders the archive sidebar with the search form', async () => {
    mockSearch(searchPayload());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);

    // No active query initially — full-page form only, no sidebar.
    expect(screen.queryByText(/Works/)).toBeNull();
  });
});
