import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  // Default to modern UI in tests (archive is the app default)
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

// Mock SvelteKit modules (same pattern as the /search page.test.ts).
vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ url: new URL('http://localhost/ask') });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

async function loadAskPage() {
  return await import('./+page.svelte');
}

/** A canned translated response from POST /api/search/ask. */
function translatedResponse(overrides: Record<string, unknown> = {}) {
  return {
    total: 2,
    page: 1,
    per_page: 20,
    results: [
      {
        url_id: 'fic-1',
        title: 'The Dark Lord Ascendant',
        author: 'Author One',
        source: 'archiveofourown.org',
        words: 52341,
        chapters: 22,
        status: 'complete',
        description: 'Harry becomes something darker.',
        updated: '2026-01-01T00:00:00Z',
        rank: 1,
        snippet: null,
        tags: [
          { name: 'Harry Potter', type: 'Character', type_id: 2, score: 8 },
          { name: 'Dark Harry Potter', type: 'Freeform', type_id: 4, score: 7 },
        ],
        total_freeform: 1,
        comment_count: 4,
        kudos_count: 9,
      },
      {
        url_id: 'fic-2',
        title: 'A Second Chance',
        author: 'Author Two',
        source: 'archiveofourown.org',
        words: 61000,
        chapters: 31,
        status: 'complete',
        description: 'Time travel fixes everything.',
        updated: '2026-02-01T00:00:00Z',
        rank: 2,
        snippet: null,
        tags: [],
        total_freeform: 0,
        comment_count: 0,
        kudos_count: 2,
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
    translated: true,
    nl_query: 'dark harry potter completed over 50k',
    applied_params: 'harry potter status:complete words:>50k',
    ...overrides,
  };
}

describe('AskPage', () => {
  it('renders the ask textarea and submit button', async () => {
    const { default: AskPage } = await loadAskPage();
    render(AskPage);

    expect(screen.getByLabelText(/natural language search query/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Ask' })).toBeTruthy();
  });

  it('posts the query to /api/search/ask and renders results with the Interpreted as chip', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => translatedResponse(),
    });

    const { default: AskPage } = await loadAskPage();
    render(AskPage);

    const input = screen.getByLabelText(/natural language search query/i);
    await fireEvent.input(input, { target: { value: 'dark harry potter completed over 50k' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Ask' }));

    await waitFor(() => {
      expect(screen.getByText('The Dark Lord Ascendant')).toBeTruthy();
    });

    // The POST body carries the NL query.
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/search/ask');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toEqual({ q: 'dark harry potter completed over 50k' });

    // Interpreted-as chip shows the applied v2 query.
    expect(screen.getByText(/Interpreted as:/)).toBeTruthy();
    // Archive UI: the chip is .archive-chip without a title attr.
    const archiveChip = screen.getByText(/harry potter status:complete words:>50k/);
    expect(archiveChip).toBeTruthy();

    // Both results render with metadata.
    expect(screen.getByText('A Second Chance')).toBeTruthy();
    // Archive UI: WorkBlurb renders "Words: 52,341" (dt/dd), not inline.
    expect(screen.getByText(/52,341/)).toBeTruthy();

    // Translated ⇒ no plain-search notice.
    expect(screen.queryByText(/plain search \(no translation\)/)).toBeNull();
  });

  it('shows the plain-search notice when translation was unavailable', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () =>
        translatedResponse({
          translated: false,
          applied_params: 'lighthouse pastry',
        }),
    });

    const { default: AskPage } = await loadAskPage();
    render(AskPage);

    const input = screen.getByLabelText(/natural language search query/i);
    await fireEvent.input(input, { target: { value: 'lighthouse pastry' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Ask' }));

    await waitFor(() => {
      expect(screen.getByText(/plain search \(no translation\)/)).toBeTruthy();
    });
    // The chip still shows the applied q.
    expect(screen.getByText(/lighthouse pastry/)).toBeTruthy();
  });

  it('shows the error card when the request fails', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      text: async () => 'boom',
    });

    const { default: AskPage } = await loadAskPage();
    render(AskPage);

    const input = screen.getByLabelText(/natural language search query/i);
    await fireEvent.input(input, { target: { value: 'anything' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Ask' }));

    await waitFor(() => {
      expect(screen.getByText(/Ask failed \(500\): boom/)).toBeTruthy();
    });
  });

  it('shows "no results" when the search returns zero fics', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => translatedResponse({ total: 0, results: [] }),
    });

    const { default: AskPage } = await loadAskPage();
    render(AskPage);

    const input = screen.getByLabelText(/natural language search query/i);
    await fireEvent.input(input, { target: { value: 'nothing matches this' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Ask' }));

    await waitFor(() => {
      expect(screen.getByText(/No results found\. Try a different ask\./)).toBeTruthy();
    });
  });

  it('offers to turn an empty ask into a fic request (Ask × Requests combine)', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => translatedResponse({ total: 0, results: [] }),
    });

    const { default: AskPage } = await loadAskPage();
    render(AskPage);

    const input = screen.getByLabelText(/natural language search query/i);
    await fireEvent.input(input, { target: { value: 'dark harry, no bashing' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Ask' }));

    await waitFor(() => {
      expect(screen.getByText(/Turn this ask into a fic request/)).toBeTruthy();
    });

    const link = screen.getByRole('link', { name: /Turn this into a request/ });
    expect(link.getAttribute('href')).toBe(
      '/requests/new?q=dark%20harry%2C%20no%20bashing'
    );
  });
});
