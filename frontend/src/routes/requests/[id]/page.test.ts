import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { page } from '$app/stores';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function requestDetail(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    request: {
      id: 1,
      user_id: 100,
      title: 'Recs for slow burn',
      body: 'something with a slow burn romance',
      seed_work_id: null,
      seed_fic: null,
      status: 'open',
      created_at: '2026-08-08T00:00:00Z',
      accepted_answer_id: null,
      upvotes: 3,
      my_upvote: false,
    },
    answers: [],
    ...overrides,
  };
}

function workAnswer(overrides: Record<string, unknown> = {}) {
  return {
    id: 10,
    user_id: 5,
    username: 'ficseeker',
    work_id: 42,
    fic_title: 'The Slow Burn Chronicles',
    fic_author: 'Author A',
    pitch: 'Slow burn done right',
    source: 'archiveofourown.org',
    created_at: '2026-08-09T00:00:00Z',
    score: 5,
    my_vote: null,
    accepted: false,
    answer_kind: 'work',
    search_query: null,
    payload: null,
    ...overrides,
  };
}

function llmAnswer(overrides: Record<string, unknown> = {}) {
  return {
    id: 11,
    user_id: 1,
    username: 'archivist',
    work_id: null,
    fic_title: '',
    fic_author: '',
    pitch: 'Try "Heir to the Night" — it has a deliberate slow-burn romance with plenty of tension.',
    source: '',
    created_at: '2026-08-09T00:00:00Z',
    score: 3,
    my_vote: null,
    accepted: false,
    answer_kind: 'llm',
    search_query: null,
    payload: { url_id: 99 },
    ...overrides,
  };
}

function searchAnswer(overrides: Record<string, unknown> = {}) {
  return {
    id: 12,
    user_id: 7,
    username: 'searcher',
    work_id: null,
    fic_title: '',
    fic_author: '',
    pitch: 'A focused query narrows it down',
    source: '',
    created_at: '2026-08-09T00:00:00Z',
    score: 2,
    my_vote: null,
    accepted: false,
    answer_kind: 'search',
    search_query: 'slow burn completed over 50k',
    payload: { params: { q: 'slow burn', min_words: 50000, complete: true } },
    ...overrides,
  };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  // Default to modern UI in tests (archive is the app default)
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));

  // Mock auth store — no logged-in user so myId stays null
  vi.doMock('$lib/stores/auth.svelte', () => ({
    auth: {
      isLoggedIn: false,
      user: null,
      subscribe: (fn: Function) => {
        fn({ isLoggedIn: false, user: null });
        return () => {};
      },
    },
  }));
});

// Mock SvelteKit modules
vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ params: { id: '1' }, url: new URL('http://localhost/requests/1') });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

describe('request detail page — answer rendering by kind', () => {
  it('renders a work answer as a link to /works/{work_id}', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      return Promise.resolve(jsonResponse(requestDetail({ answers: [workAnswer()] })));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('The Slow Burn Chronicles')).toBeTruthy();
    });

    const link = screen.getByText('The Slow Burn Chronicles').closest('a');
    expect(link).toBeTruthy();
    expect(link?.getAttribute('href')).toBe('/works/42');
    expect(screen.getByText(/by Author A/)).toBeTruthy();
    expect(screen.getByText(/Slow burn done right/)).toBeTruthy();
    expect(screen.getByText(/ficseeker/)).toBeTruthy();
  });

  it('renders an llm answer as a FicNexus Archivist prose block', async () => {
    mockFetch.mockImplementation(() =>
      Promise.resolve(jsonResponse(requestDetail({ answers: [llmAnswer()] })))
    );

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('FicNexus Archivist')).toBeTruthy();
    });

    // Archive UI: no separate "Archivist answer" label; attribution span + pitch cover it.
    expect(
      screen.getByText(/Try "Heir to the Night" — it has a deliberate slow-burn romance/)
    ).toBeTruthy();
    expect(screen.getByText(/archivist/)).toBeTruthy();
  });

  it('renders a search answer as a "Try this search" chip linking to /search', async () => {
    mockFetch.mockImplementation(() =>
      Promise.resolve(jsonResponse(requestDetail({ answers: [searchAnswer()] })))
    );

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Try this search/)).toBeTruthy();
    });

    const link = screen.getByText(/Try this search/).closest('a');
    expect(link).toBeTruthy();
    // buildSearchQuery uses URLSearchParams -> space='+' not '%20'
    expect(decodeURIComponent(link?.getAttribute('href')?.replace(/\+/g, '%20') ?? '')).toContain('slow burn completed over 50k');
    // Archive UI: query text is inside the chip link, not a separate span.
    expect(screen.getByText(/slow burn completed over 50k/)).toBeTruthy();
  });

  it('does not render work-specific fields (fic_title link / author line) for non-work answers', async () => {
    mockFetch.mockImplementation(() =>
      Promise.resolve(
        jsonResponse(requestDetail({ answers: [llmAnswer(), searchAnswer()] }))
      )
    );

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Try this search/)).toBeTruthy();
    });

    // No fic_title link should appear for llm/search answers
    expect(screen.queryByText('The Slow Burn Chronicles')).toBeNull();
  });

  it('renders mixed answer kinds in the correct typed form', async () => {
    mockFetch.mockImplementation(() =>
      Promise.resolve(
        jsonResponse(
          requestDetail({
            answers: [
              workAnswer({ id: 1, fic_title: 'Work One', fic_author: 'Auth One' }),
              llmAnswer({ id: 2, pitch: 'LLM suggested fic here' }),
              searchAnswer({ id: 3, search_query: 'query one two' }),
            ],
          })
        )
      )
    );

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Work One')).toBeTruthy();
    });
    expect(screen.getByText('FicNexus Archivist')).toBeTruthy();
    expect(screen.getByText(/Try this search/)).toBeTruthy();
    expect(screen.getByText(/Auth One/)).toBeTruthy();
  });
});
