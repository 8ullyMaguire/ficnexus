import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function seriesPayload() {
  return jsonResponse({
    err: 0,
    series: {
      id: 7,
      name: 'The Test Trilogy',
      description: 'Three connected stories.',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
      work_count: 3,
    },
    works: [
      {
        work_id: 1,
        url_id: 'fic-a',
        canonical_title: 'Alpha',
        canonical_author: 'Author A',
        title: 'Alpha',
        author: 'Author A',
        words: 10000,
        chapters: 5,
        status: 'complete',
        next_in_series: { work_id: 2, canonical_title: 'Beta', url_id: 'fic-b' },
      },
      {
        work_id: 2,
        url_id: 'fic-b',
        canonical_title: 'Beta',
        canonical_author: 'Author A',
        title: 'Beta',
        author: 'Author A',
        words: 20000,
        chapters: 8,
        status: 'complete',
        next_in_series: { work_id: 3, canonical_title: 'Gamma', url_id: 'fic-c' },
      },
      {
        work_id: 3,
        url_id: 'fic-c',
        canonical_title: 'Gamma',
        canonical_author: 'Author A',
        title: 'Gamma',
        author: 'Author A',
        words: 30000,
        chapters: 10,
        status: 'complete',
        next_in_series: null,
      },
    ],
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  mockFetch.mockResolvedValue(seriesPayload());
});

async function loadPage() {
  return await import('./+page.svelte');
}

describe('series page', () => {
  it('renders the series title, work count, and ordered works', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { id: '7' } } });

    await waitFor(() => {
      expect(screen.getByText('The Test Trilogy')).toBeTruthy();
    });
    expect(screen.getByText('Three connected stories.')).toBeTruthy();
    expect(screen.getByText('3 works in series')).toBeTruthy();

    // Ordered works with position numbers and fic links
    expect(screen.getByText('Alpha')).toBeTruthy();
    expect(screen.getByText('Beta')).toBeTruthy();
    expect(screen.getByText('Gamma')).toBeTruthy();
    const alphaLink = screen.getByRole('link', { name: /Alpha/ });
    expect(alphaLink.getAttribute('href')).toBe('/works/fic-a');
  });

  it('highlights next-in-series: each non-final work shows its successor', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { id: '7' } } });

    await waitFor(() => {
      expect(screen.getByText('The Test Trilogy')).toBeTruthy();
    });

    // Alpha's card advertises Beta as next in series; Beta's card Gamma.
    expect(screen.getByText(/Next in series: Beta/)).toBeTruthy();
    expect(screen.getByText(/Next in series: Gamma/)).toBeTruthy();
    // The final work has no next — no badge on the Gamma card.
    const nextBadges = screen.getAllByText(/Next in series:/);
    expect(nextBadges.length).toBe(2);
  });

  it('links next-in-series work through its own fic page', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { id: '7' } } });

    await waitFor(() => {
      expect(screen.getByText('The Test Trilogy')).toBeTruthy();
    });

    // The Beta card link is the one whose href points at fic-b (the
    // next-in-series badge also contains "Beta", so pick the fic link).
    const betaLink = Array.from(document.querySelectorAll('a.archive-list-title')).find(
      (a) => a.getAttribute('href') === '/works/fic-b',
    );
    expect(betaLink).toBeTruthy();
    expect(betaLink!.textContent).toContain('Beta');
  });
});
