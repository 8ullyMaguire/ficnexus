import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function authorPayload() {
  return jsonResponse({
    err: 0,
    author: {
      name: 'Test Author',
      work_count: 2,
      total_words: 30000,
      top_tags: [
        { name: 'Angst', tag_type_id: 4, usage_count: 2 },
        { name: 'Harry Potter', tag_type_id: 1, usage_count: 1 },
      ],
    },
    works: [
      {
        work_id: 10,
        canonical_title: 'The First Story',
        description: 'A gripping tale.',
        canonical_author: 'Test Author',
        url_id: 'fic-10',
        words: 10000,
        chapters: 4,
        status: 'complete',
      },
      {
        work_id: 11,
        canonical_title: 'The Second Story',
        description: 'Even better.',
        canonical_author: 'Test Author',
        url_id: 'fic-11',
        words: 20000,
        chapters: 9,
        status: 'ongoing',
      },
    ],
    orphans: [],
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  mockFetch.mockResolvedValue(authorPayload());
});

async function loadPage() {
  return await import('./+page.svelte');
}

describe('author page', () => {
  it('renders the author name, stats, and works grid', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { name: 'Test Author' } } });

    await waitFor(() => {
      expect(screen.getByText('✍️ Test Author')).toBeTruthy();
    });
    expect(screen.getByText('2 works')).toBeTruthy();
    expect(screen.getByText('30,000 words')).toBeTruthy();
    expect(screen.getByText('The First Story')).toBeTruthy();
    expect(screen.getByText('The Second Story')).toBeTruthy();
  });

  it('links each work card to its fic page via /works/{url_id}', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { name: 'Test Author' } } });

    await waitFor(() => {
      expect(screen.getByText('✍️ Test Author')).toBeTruthy();
    });
    const firstLink = screen.getByRole('link', { name: /The First Story/ });
    expect(firstLink.getAttribute('href')).toBe('/works/fic-10');
    const secondLink = screen.getByRole('link', { name: /The Second Story/ });
    expect(secondLink.getAttribute('href')).toBe('/works/fic-11');
  });

  it('renders top tags as search chips', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { name: 'Test Author' } } });

    await waitFor(() => {
      expect(screen.getByText('✍️ Test Author')).toBeTruthy();
    });
    const chip = screen.getByRole('link', { name: 'Angst' });
    expect(chip.getAttribute('href')).toBe('/search?q=Angst');
  });
});
