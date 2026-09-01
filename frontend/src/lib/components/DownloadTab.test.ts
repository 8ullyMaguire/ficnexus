import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  // These tests exercise the modern-mode markup (a.dl-btn etc.); pin uiMode
  // so the default ('archive') branch doesn't render instead.
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

// Import the component lazily so $lib alias resolves after build.
async function loadDownloadTab() {
  return await import('$lib/components/DownloadTab.svelte');
}

describe('DownloadTab', () => {
  it('shows an error when URL is empty', async () => {
    const { default: DownloadTab } = await loadDownloadTab();
    render(DownloadTab);
    await fireEvent.click(screen.getByText('Download'));
    expect(screen.getByText(/paste a fanfiction URL/i)).toBeTruthy();
  });

  it('renders download links on success', async () => {
    const { default: DownloadTab } = await loadDownloadTab();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'abc123',
        meta: {
          id: 'abc123',
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
          author_url: 'https://archiveofourown.org/users/Author',
          author_local_id: 'a1',
          source_id: 1,
          author_id: 2,
        },
        epub_url: '/cache/epub/abc123?h=xyz',
        html_url: '/cache/html/abc123?h=html1',
      }),
    });
    render(DownloadTab);
    const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: 'https://archiveofourown.org/works/1' } });
    await fireEvent.click(screen.getByText('Download'));
    await waitFor(() => expect(screen.getByText('My Story')).toBeTruthy());
    // The label text now appears both as a dropdown option and a button, so
    // assert the download links specifically.
    expect(screen.getByRole('link', { name: 'EPUB' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'HTML' })).toBeTruthy();
  });

  it('auto-downloads a URL prefilled via sessionStorage (home dashboard handoff)', async () => {
    const { default: DownloadTab } = await loadDownloadTab();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        url_id: 'rr123',
        meta: {
          id: 'rr123',
          title: 'Gifted',
          author: 'Ellake',
          chapters: 30,
          words: 0,
          description: 'desc',
          status: 'ongoing',
          source: 'https://www.royalroad.com/fiction/181303/gifted',
          created: '2024-01-01T00:00:00Z',
          updated: '2024-01-02T00:00:00Z',
          extra_meta: null,
          raw_extended_meta: null,
          author_url: '',
          author_local_id: '181303',
          source_id: 6,
          author_id: 0,
        },
        epub_url: '/cache/epub/6_181303?h=abc',
        html_url: '/cache/html/6_181303?h=def',
        txt_url: '/cache/txt/6_181303?h=ghi',
      }),
    });

    const pasted = 'https://www.royalroad.com/fiction/181303/gifted?utm_source=home&utm_medium=rising-stars';
    sessionStorage.setItem('fichub_dl_url', pasted);
    render(DownloadTab);

    await waitFor(() => expect(screen.getByText('Gifted')).toBeTruthy());
    expect(screen.getByRole('link', { name: 'EPUB' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'TXT' })).toBeTruthy();
    expect(sessionStorage.getItem('fichub_dl_url')).toBeNull();
  });
});

// ─── All export formats + preferred-format selector ────────────────────────

describe('DownloadTab all formats + preferred-format selector', () => {
  const meta = {
    id: 'all123',
    work_id: 7,
    title: 'Every Format',
    author: 'Author',
    chapters: 3,
    words: 1000,
    description: 'desc',
    status: 'complete',
    source: 'https://archiveofourown.org/works/7',
    created: '2024-01-01T00:00:00Z',
    updated: '2024-01-02T00:00:00Z',
    extra_meta: null,
    raw_extended_meta: null,
    author_url: 'https://archiveofourown.org/users/Author',
    author_local_id: 'a7',
    source_id: 1,
    author_id: 2,
  };

  function allFormatsPayload() {
    return {
      err: 0,
      url_id: 'all123',
      meta,
      epub_url: '/cache/epub/all123?h=1',
      mobi_url: '/cache/mobi/all123?h=2',
      azw3_url: '/cache/azw3/all123?h=3',
      pdf_url: '/cache/pdf/all123?h=4',
      docx_url: '/cache/docx/all123?h=5',
      kepub_url: '/cache/kepub/all123?h=6',
      fb2_url: '/cache/fb2/all123?h=7',
      html_url: '/cache/html/all123?h=8',
      txt_url: '/cache/txt/all123?h=9',
      md_url: '/cache/md/all123?h=10',
    };
  }

  function mockExport(payload: unknown) {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/epub?q=')) return Promise.resolve({ ok: true, json: async () => payload });
      return Promise.resolve({ ok: true, json: async () => ({ err: 0, bookmarks: [] }) });
    });
  }

  async function downloadStory(url = 'https://ao3.org/works/7') {
    const { default: DownloadTab } = await loadDownloadTab();
    render(DownloadTab);
    const input = screen.getByLabelText('Fanfiction URL') as HTMLInputElement;
    await fireEvent.input(input, { target: { value: url } });
    await fireEvent.click(screen.getByText('Download'));
    await waitFor(() => expect(screen.getByText('Every Format')).toBeTruthy());
  }

  function dlLinks(): HTMLElement[] {
    // Archive UI: download links use .archive-btn (modern's .dl-btn is gone).
    return Array.from(document.querySelectorAll<HTMLElement>('a.archive-btn'));
  }

  it('renders download links for all ten formats when their URLs are present', async () => {
    mockExport(allFormatsPayload());
    await downloadStory();

    for (const label of ['EPUB', 'MOBI', 'AZW3', 'PDF', 'DOCX', 'KEPUB', 'FB2', 'HTML', 'TXT', 'MD']) {
      expect(screen.getByRole('link', { name: label })).toBeTruthy();
    }
    expect(screen.getByRole('link', { name: 'DOCX' }).getAttribute('href')).toContain('/cache/docx/');
    expect(screen.getByRole('link', { name: 'FB2' }).getAttribute('href')).toContain('/cache/fb2/');
    expect(screen.getByRole('link', { name: 'KEPUB' }).getAttribute('href')).toContain('/cache/kepub/');
  });

  it('highlights the preferred format button and sorts it first', async () => {
    localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive', defaultFormat: 'mobi' }));
    mockExport(allFormatsPayload());
    await downloadStory();

    const links = dlLinks();
    expect(links[0].textContent).toBe('MOBI');
    expect(links[0].classList.contains('preferred')).toBe(true);
    expect(links.some((l) => l.classList.contains('preferred'))).toBe(true);
  });

  it('falls back to the first available format when the stored pref is unavailable', async () => {
    // Store a pref for a format the result does not provide.
    localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive', defaultFormat: 'docx' }));
    mockExport({
      ...allFormatsPayload(),
      docx_url: null,
    });
    await downloadStory();

    const links = dlLinks();
    expect(links[0].textContent).toBe('EPUB');
    expect(links[0].classList.contains('preferred')).toBe(true);
    // The stored pref is not clobbered.
    expect(JSON.parse(localStorage.getItem('fichub_prefs_v1') || '{}').defaultFormat).toBe('docx');
  });

  it('updates the localStorage pref when a format is chosen in the selector', async () => {
    mockExport(allFormatsPayload());
    await downloadStory();

    const select = screen.getByLabelText('Preferred format') as HTMLSelectElement;
    expect(select.options.length).toBe(10);
    await fireEvent.change(select, { target: { value: 'pdf' } });

    expect(JSON.parse(localStorage.getItem('fichub_prefs_v1') || '{}').defaultFormat).toBe('pdf');
    // The preferred button follows the selector and moves to the front.
    const links = dlLinks();
    expect(links[0].textContent).toBe('PDF');
    expect(links[0].classList.contains('preferred')).toBe(true);
  });

  it('sets the preferred format when a download button is clicked', async () => {
    mockExport(allFormatsPayload());
    await downloadStory();

    await fireEvent.click(screen.getByRole('link', { name: 'KEPUB' }));
    expect(JSON.parse(localStorage.getItem('fichub_prefs_v1') || '{}').defaultFormat).toBe('kepub');
    const links = dlLinks();
    expect(links[0].textContent).toBe('KEPUB');
    expect(links[0].classList.contains('preferred')).toBe(true);
  });

  it('shows lazy convert buttons for Calibre formats missing from the response', async () => {
    mockExport({
      ...allFormatsPayload(),
      mobi_url: null,
      pdf_url: null,
      azw3_url: null,
    });
    await downloadStory();

    // Lazy formats still appear, as buttons (⚡), not links.
    const mobiBtn = screen.getByRole('button', { name: /MOBI/ });
    const pdfBtn = screen.getByRole('button', { name: /PDF/ });
    const azw3Btn = screen.getByRole('button', { name: /AZW3/ });
    expect(mobiBtn).toBeTruthy();
    expect(pdfBtn).toBeTruthy();
    expect(azw3Btn).toBeTruthy();
    // Ready formats are still links.
    expect(screen.getByRole('link', { name: 'EPUB' })).toBeTruthy();
    expect(screen.getByRole('link', { name: 'DOCX' })).toBeTruthy();
  });

  it('converts a lazy format on click and turns it into a download link', async () => {
    mockExport({ ...allFormatsPayload(), mobi_url: null });
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/epub?q=')) return Promise.resolve({ ok: true, json: async () => ({ ...allFormatsPayload(), mobi_url: null }) });
      if (url.includes('/api/epub/convert')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({ err: 0, url_id: 'all123', format: 'mobi', hash: 'conv1', url: '/cache/mobi/all123?h=conv1', cached: false }),
        });
      }
      return Promise.resolve({ ok: true, json: async () => ({ err: 0, bookmarks: [] }) });
    });
    await downloadStory();

    const mobiBtn = screen.getByRole('button', { name: /MOBI/ });
    await fireEvent.click(mobiBtn);

    // After conversion, the button becomes a real download link.
    await waitFor(() => expect(screen.getByRole('link', { name: 'MOBI' })).toBeTruthy());
    expect(screen.getByRole('link', { name: 'MOBI' }).getAttribute('href')).toContain('/cache/mobi/');
  });
});
