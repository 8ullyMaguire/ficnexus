import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function loadRecsTab() {
  return await import('$lib/components/RecommendationsTab.svelte');
}

describe('RecommendationsTab', () => {
  it('renders search input and button', async () => {
    const { default: RecsTab } = await loadRecsTab();
    render(RecsTab);

    expect(screen.getByPlaceholderText(/fic you liked/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /recommend/i })).toBeTruthy();
  });

  it('shows error when URL is empty', async () => {
    const { default: RecsTab } = await loadRecsTab();
    render(RecsTab);

    await fireEvent.click(screen.getByRole('button', { name: /recommend/i }));

    expect(screen.getByText(/paste the url/i)).toBeTruthy();
  });

  it('fetches and displays recommendations', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'abc123',
        recommendations: [
          {
            url_id: 'rec1',
            title: 'Similar Story',
            author: 'Author1',
            words: 30000,
            chapters: 8,
            status: 'complete',
            site_domain: 'archiveofourown.org',
            summary: 'A similar story',
            score: 0.8,
            community_score: 3,
            download_urls: { epub: '/cache/epub/rec1' },
          },
          {
            url_id: 'rec2',
            title: 'Another Similar',
            author: 'Author2',
            words: 20000,
            chapters: 4,
            status: 'ongoing',
            site_domain: 'fanfiction.net',
            summary: 'Another similar story',
            score: 0.6,
            community_score: 1,
            download_urls: {},
          },
        ],
        generated_at: '2024-01-01T00:00:00Z',
      }),
    });

    const { default: RecsTab } = await loadRecsTab();
    render(RecsTab);

    const input = screen.getByPlaceholderText(/fic you liked/i);
    await fireEvent.input(input, { target: { value: 'https://archiveofourown.org/works/1' } });
    await fireEvent.click(screen.getByRole('button', { name: /recommend/i }));

    await waitFor(() => {
      expect(screen.getByText('Similar Story')).toBeTruthy();
    });
    expect(screen.getByText('Another Similar')).toBeTruthy();
    expect(screen.getByText('2 recommendations')).toBeTruthy();
  });

  it('shows error on API failure', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      text: async () => 'backend error',
    });

    const { default: RecsTab } = await loadRecsTab();
    render(RecsTab);

    const input = screen.getByPlaceholderText(/fic you liked/i);
    await fireEvent.input(input, { target: { value: 'https://archiveofourown.org/works/1' } });
    await fireEvent.click(screen.getByRole('button', { name: /recommend/i }));

    await waitFor(() => {
      expect(screen.getByText(/backend error/i)).toBeTruthy();
    });
  });

  it('EPUB download link is shown when available', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'abc123',
        recommendations: [
          {
            url_id: 'rec1',
            title: 'With EPUB',
            author: 'Author',
            words: 10000,
            chapters: 3,
            status: 'complete',
            site_domain: 'ao3',
            summary: '',
            score: 0.9,
            community_score: 0,
            download_urls: { epub: '/cache/epub/rec1?h=abc' },
          },
        ],
        generated_at: '2024-01-01T00:00:00Z',
      }),
    });

    const { default: RecsTab } = await loadRecsTab();
    render(RecsTab);

    const input = screen.getByPlaceholderText(/fic you liked/i);
    await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
    await fireEvent.click(screen.getByRole('button', { name: /recommend/i }));

    await waitFor(() => {
      expect(screen.getByText('EPUB')).toBeTruthy();
    });
  });
});
