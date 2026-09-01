import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

// DownloadTab extra paths: sessionStorage prefill auto-download, server
// error message, bookmark toggle, and comment submit (logged-in flows).
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

import { auth } from '$lib/stores/auth.svelte';

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  sessionStorage.clear();
  auth.initialized = true;
  auth.user = { id: 1, username: 'alice', role: 0, reputation: 0 };
});

async function loadTab() {
  return await import('$lib/components/DownloadTab.svelte');
}

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

const exportPayload = {
  err: 0,
  url_id: 'abc123',
  meta: {
    id: 'abc123',
    work_id: 42,
    title: 'My Story',
    author: 'Author',
    chapters: 10,
    words: 50000,
    description: '<p>desc</p>',
    status: 'complete',
    source: 'https://archiveofourown.org/works/1',
    created: '2024-01-01T00:00:00Z',
    updated: '2024-01-02T00:00:00Z',
    extra_meta: null,
    raw_extended_meta: null,
    author_url: null,
    author_local_id: null,
    source_id: null,
    author_id: null,
  },
  epub_url: '/cache/epub/abc123?h=xyz',
};

describe('DownloadTab extra paths', () => {
  it('auto-downloads a URL prefilled via sessionStorage', async () => {
    sessionStorage.setItem('fichub_dl_url', 'https://ao3.org/works/1');
    mockFetch
      .mockResolvedValueOnce(okJson(exportPayload))
      // auth-gated follow-ups (bookmarks/ratings/comments) — all resolve empty
      .mockResolvedValue(okJson({ err: 0, bookmarks: [] }))
      .mockResolvedValue(okJson({ err: 0, avg_rating: 0, rating_count: 0, review_count: 0 }))
      .mockResolvedValue(okJson({ err: 0, comments: [] }));

    const { default: Tab } = await loadTab();
    render(Tab);

    await waitFor(() => {
      expect(screen.getByText('My Story')).toBeTruthy();
    });
    const dlCall = mockFetch.mock.calls[0];
    expect(String(dlCall[0])).toContain('/api/epub?q=');
  });

  it('shows the server error message when the export errs', async () => {
    mockFetch.mockResolvedValue(okJson({ err: -1, msg: 'That URL is not supported.' }));
    const { default: Tab } = await loadTab();
    render(Tab);
    const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'https://example.com/x' } });
    await fireEvent.click(screen.getByText('Download'));
    await waitFor(() => {
      expect(screen.getByText(/that url is not supported/i)).toBeTruthy();
    });
  });

  it('toggles the bookmark on and off', async () => {
    mockFetch
      .mockResolvedValueOnce(okJson(exportPayload))
      .mockResolvedValue(okJson({ err: 0, bookmarks: [] }))
      .mockResolvedValue(okJson({ err: 0, avg_rating: 0, rating_count: 0, review_count: 0 }))
      .mockResolvedValue(okJson({ err: 0, comments: [] }))
      .mockResolvedValue(okJson({ err: 0 })) // addBookmark
      .mockResolvedValue(okJson({ err: 0 })); // removeBookmark

    const { default: Tab } = await loadTab();
    render(Tab);

    const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
    await fireEvent.click(screen.getByText('Download'));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /bookmark/i })).toBeTruthy();
    });

    await fireEvent.click(screen.getByRole('button', { name: /bookmark/i }));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /bookmarked/i })).toBeTruthy();
    });
    const addCall = mockFetch.mock.calls.find((c) => String(c[0]) === '/api/bookmarks' && c[1]?.method === 'POST');
    expect(addCall).toBeTruthy();

    await fireEvent.click(screen.getByRole('button', { name: /bookmarked/i }));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /bookmark/i })).toBeTruthy();
    });
    const delCall = mockFetch.mock.calls.find((c) => String(c[0]) === '/api/bookmarks/42' && c[1]?.method === 'DELETE');
    expect(delCall).toBeTruthy();
  });

  it('posts a comment and reloads the list', async () => {
    mockFetch
      .mockResolvedValueOnce(okJson(exportPayload))
      .mockResolvedValue(okJson({ err: 0, bookmarks: [] }))
      .mockResolvedValue(okJson({ err: 0, avg_rating: 0, rating_count: 0, review_count: 0 }))
      .mockResolvedValue(okJson({ err: 0, comments: [] }))
      .mockResolvedValue(okJson({ err: 0, comment_id: 9 })) // addComment
      .mockResolvedValue(okJson({ err: 0, comments: [{ id: 9, user: { id: 1, username: 'alice' }, body: 'Nice fic', created_at: '2026-01-01T00:00:00Z' }] })); // reload

    const { default: Tab } = await loadTab();
    render(Tab);

    const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
    await fireEvent.click(screen.getByText('Download'));
    await waitFor(() => {
      expect(screen.getByText('My Story')).toBeTruthy();
    });

    const textarea = screen.getByPlaceholderText(/write a comment/i) as HTMLTextAreaElement;
    await fireEvent.input(textarea, { target: { value: 'Nice fic' } });
    await fireEvent.click(screen.getByRole('button', { name: /post comment/i }));
    await waitFor(() => {
      expect(screen.getByText('Nice fic')).toBeTruthy();
    });
  });
});
