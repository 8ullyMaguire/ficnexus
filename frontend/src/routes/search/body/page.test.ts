import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

// Mock SvelteKit modules (same pattern as the /search page.test.ts).
vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ url: new URL('http://localhost/search/body') });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

async function loadBodySearchPage() {
  return await import('./+page.svelte');
}

/** A canned response from GET /api/search/body. */
function bodySearchResponse(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    total: 2,
    page: 1,
    per_page: 20,
    results: [
      {
        url_id: 'fic-1',
        title: 'The Wandless Art',
        author: 'Author One',
        source: 'archiveofourown.org',
        words: 52341,
        chapters: 22,
        status: 'complete',
        description: 'Harry learns to cast without a wand.',
        work_id: 11,
        body_snippet:
          '…orchard, practicing <mark>wandless</mark> magic far from the prying eyes of Privet Drive…',
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
        work_id: 12,
        body_snippet: null,
      },
    ],
    ...overrides,
  };
}

describe('BodySearchPage', () => {
  it('renders the search box and the link back to advanced search', async () => {
    const { default: BodySearchPage } = await loadBodySearchPage();
    render(BodySearchPage);

    expect(screen.getByLabelText(/body search query/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /search/i })).toBeTruthy();
    const back = screen.getByRole('link', { name: /advanced search/i });
    expect(back.getAttribute('href')).toBe('/search');
  });

  it('queries /api/search/body and renders results with the highlighted snippet', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => bodySearchResponse(),
    });

    const { default: BodySearchPage } = await loadBodySearchPage();
    render(BodySearchPage);

    const input = screen.getByLabelText(/body search query/i);
    await fireEvent.input(input, { target: { value: 'wandless magic' } });
    await fireEvent.click(screen.getByRole('button', { name: /search/i }));

    await waitFor(() => {
      expect(screen.getByText('The Wandless Art')).toBeTruthy();
    });

    // The request went to the body-search endpoint with the right params.
    const [url] = mockFetch.mock.calls[0];
    expect(url).toContain('/api/search/body?');
    expect(url).toContain('q=wandless+magic');
    expect(url).toContain('page=1');
    expect(url).toContain('per_page=20');

    // Highlighted snippet renders (with the <mark> from the server).
    const snippet = screen.getByText(/orchard, practicing/i);
    expect(snippet.innerHTML).toContain('<mark>wandless</mark>');

    // Second result (no snippet) falls back to the description.
    expect(screen.getByText('A Second Chance')).toBeTruthy();
    expect(screen.getByText('Time travel fixes everything.')).toBeTruthy();
    // Count line shows the total.
    expect(screen.getByText(/2 fics matched/)).toBeTruthy();
  });

  it('shows "no results" when the search returns zero fics', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => bodySearchResponse({ total: 0, results: [] }),
    });

    const { default: BodySearchPage } = await loadBodySearchPage();
    render(BodySearchPage);

    const input = screen.getByLabelText(/body search query/i);
    await fireEvent.input(input, { target: { value: 'nothing matches' } });
    await fireEvent.click(screen.getByRole('button', { name: /search/i }));

    await waitFor(() => {
      expect(screen.getByText(/No results found/)).toBeTruthy();
    });
  });

  it('shows the error card when the request fails', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
      text: async () => 'boom',
    });

    const { default: BodySearchPage } = await loadBodySearchPage();
    render(BodySearchPage);

    const input = screen.getByLabelText(/body search query/i);
    await fireEvent.input(input, { target: { value: 'anything' } });
    await fireEvent.click(screen.getByRole('button', { name: /search/i }));

    await waitFor(() => {
      expect(screen.getByText(/Body search failed \(500\): boom/)).toBeTruthy();
    });
  });
});
