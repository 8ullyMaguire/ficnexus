import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ url: new URL('http://localhost/') });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

describe('Integration: Download Format Coverage', () => {
  it('DownloadTab shows all 7 format buttons when all URLs are present', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'abc123',
        meta: {
          id: 'abc123',
          title: 'Manacled',
          author: 'senlinyu',
          chapters: 77,
          words: 370515,
          description: '<p>Harry Potter is dead...</p>',
          status: 'complete',
          source: 'https://archiveofourown.org/works/14454174',
          created: '2018-04-27T00:00:00Z',
          updated: '2019-08-19T00:00:00Z',
          extra_meta: null,
          raw_extended_meta: null,
          author_url: 'https://archiveofourown.org/users/senlinyu',
          author_local_id: 'senlinyu',
          source_id: 1,
          author_id: 123,
        },
        epub_url: '/cache/epub/abc123/abc123.epub',
        html_url: '/cache/html/abc123/abc123.zip',
        txt_url: '/cache/txt/abc123/abc123.txt',
        md_url: '/cache/md/abc123/abc123.md',
        mobi_url: '/cache/mobi/abc123/abc123.mobi',
        pdf_url: '/cache/pdf/abc123/abc123.pdf',
        azw3_url: '/cache/azw3/abc123/abc123.azw3',
        hashes: {
          epub: 'abc123',
          html: 'html1',
          txt: 'txt1',
          md: 'md1',
          mobi: 'mobi1',
          pdf: 'pdf1',
          azw3: 'azw1',
        },
        notes: [],
      }),
    });

    const { default: DownloadTab } = await import('$lib/components/DownloadTab.svelte');
    render(DownloadTab);

    const input = screen.getByLabelText('Fanfiction URL');
    await fireEvent.input(input, { target: { value: 'https://archiveofourown.org/works/14454174' } });
    await fireEvent.click(screen.getByText('Download'));

    await waitFor(() => {
      expect(screen.getByText('Manacled')).toBeTruthy();
    });

    // Verify all 7 format buttons (as links — the dropdown options also contain
    // the format labels, so scope to role=link)
    expect(screen.getByRole('link', { name: 'EPUB' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'HTML' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'TXT' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'MD' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'MOBI' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'PDF' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'AZW3' })).toBeTruthy();

    // Verify download links
    const epubLink = screen.getByRole('link', { name: 'EPUB' });
    expect(epubLink.getAttribute('href')).toBe('/cache/epub/abc123/abc123.epub');

    const txtLink = screen.getByRole('link', { name: 'TXT' });
    expect(txtLink.getAttribute('href')).toBe('/cache/txt/abc123/abc123.txt');
  });

  it('DownloadTab shows only available formats', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'def456',
        meta: {
          id: 'def456',
          title: 'Short Story',
          author: 'Author',
          chapters: 1,
          words: 5000,
          description: '<p>A short story</p>',
          status: 'complete',
          source: 'https://fanfiction.net/s/123',
          created: '2024-01-01T00:00:00Z',
          updated: '2024-01-01T00:00:00Z',
          extra_meta: null,
          raw_extended_meta: null,
          author_url: '',
          author_local_id: '',
          source_id: 2,
          author_id: 1,
        },
        epub_url: '/cache/epub/def456/def456.epub',
        html_url: null,
        txt_url: '/cache/txt/def456/def456.txt',
        md_url: null,
        mobi_url: null,
        pdf_url: null,
        azw3_url: null,
        hashes: { epub: 'abc', txt: 'def' },
        notes: [],
      }),
    });

    const { default: DownloadTab } = await import('$lib/components/DownloadTab.svelte');
    render(DownloadTab);

    const input = screen.getByLabelText('Fanfiction URL');
    await fireEvent.input(input, { target: { value: 'https://fanfiction.net/s/123' } });
    await fireEvent.click(screen.getByText('Download'));

    await waitFor(() => {
      expect(screen.getByText('Short Story')).toBeTruthy();
    });

    // Only EPUB and TXT should be present (as links; dropdown options contain
    // the same labels, so scope to role=link)
    expect(screen.getByRole('link', { name: 'EPUB' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'TXT' })).toBeTruthy();
    expect(screen.queryByRole('link', { name: 'HTML' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'MD' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'MOBI' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'PDF' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'AZW3' })).toBeNull();
  });

  it('error handling: API returns error', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: -5,
        msg: 'Unsupported URL format',
      }),
    });

    const { default: DownloadTab } = await import('$lib/components/DownloadTab.svelte');
    render(DownloadTab);

    const input = screen.getByLabelText('Fanfiction URL');
    await fireEvent.input(input, { target: { value: 'https://example.com/story' } });
    await fireEvent.click(screen.getByText('Download'));

    await waitFor(() => {
      expect(screen.getByText(/unsupported url/i)).toBeTruthy();
    });
  });

  it('error handling: network failure', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'));

    const { default: DownloadTab } = await import('$lib/components/DownloadTab.svelte');
    render(DownloadTab);

    const input = screen.getByLabelText('Fanfiction URL');
    await fireEvent.input(input, { target: { value: 'https://ao3.org/works/1' } });
    await fireEvent.click(screen.getByText('Download'));

    await waitFor(() => {
      expect(screen.getByText(/network error/i)).toBeTruthy();
    });
  });
});
