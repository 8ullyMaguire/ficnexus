import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('admin/blacklist page', () => {
  it('renders fic + author blacklist entries from the API', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        fics: [
          { url_id: 'https://archiveofourown.org/works/111', reason: 5, created: '2026-08-01T00:00:00Z' },
          { url_id: 'https://fanfiction.net/s/222', reason: 6, created: '2026-08-02T00:00:00Z' },
        ],
        authors: [
          { source_id: 1, author_id: 42, reason: 5, created: '2026-08-03T00:00:00Z' },
          { source_id: 2, author_id: 7, reason: 6, created: '2026-08-04T00:00:00Z' },
        ],
      }),
    });

    const { default: BlacklistPage } = await import('./+page.svelte');
    render(BlacklistPage);

    await waitFor(() => {
      expect(screen.getByText('https://archiveofourown.org/works/111')).toBeTruthy();
    });
    expect(screen.getByText('https://fanfiction.net/s/222')).toBeTruthy();
    // Author rows render as <source>/<author_id>.
    expect(screen.getAllByText('ao3/42')[0]).toBeTruthy();
    expect(screen.getAllByText('ffn/7').length).toBeGreaterThan(0);
    // Reason labels (one per select + one per table row each).
    expect(screen.getAllByText('5 · hard block').length).toBeGreaterThanOrEqual(2);
    expect(screen.getAllByText('6 · greylist').length).toBeGreaterThanOrEqual(2);
  });

  it('adds a fic via POST /api/admin/blacklist/fic and shows confirmation', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, fics: [], authors: [] }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, msg: 'fic blacklisted', url_id: 'https://archiveofourown.org/works/333', reason: 6 }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, fics: [], authors: [] }) });

    const { default: BlacklistPage } = await import('./+page.svelte');
    render(BlacklistPage);

    const input = await waitFor(() => screen.getByPlaceholderText(/url_id/));
    await fireEvent.input(input, { target: { value: 'https://archiveofourown.org/works/333' } });

    // Default fic reason is 5; pick greylist (6) from the fic select.
    const ficReasonSelect = screen.getByLabelText(/fic blacklist reason/i);
    await fireEvent.change(ficReasonSelect, { target: { value: '6' } });

    const addBtn = screen.getByRole('button', { name: /Blacklist fic/i });
    await fireEvent.click(addBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/blacklist/fic');
    expect(postCall[1].credentials).toBe('include');
    expect(JSON.parse(postCall[1].body as string)).toEqual({
      url_id: 'https://archiveofourown.org/works/333',
      reason: 6,
    });

    await waitFor(() => {
      expect(screen.getByText(/Blacklisted https:\/\/archiveofourown.org\/works\/333/)).toBeTruthy();
    });
  });

  it('adds an author via POST /api/admin/blacklist/author (source + author id + reason)', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, fics: [], authors: [] }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, msg: 'author blacklisted', source_id: 2, author_id: 99, reason: 5 }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, fics: [], authors: [] }) });

    const { default: BlacklistPage } = await import('./+page.svelte');
    render(BlacklistPage);

    // Pick ffn (source_id 2) from the source select.
    const sourceSelect = await waitFor(() => screen.getByLabelText(/source platform/i));
    await fireEvent.change(sourceSelect, { target: { value: '2' } });

    const authorInput = screen.getByPlaceholderText(/author_id/i);
    await fireEvent.input(authorInput, { target: { value: '99' } });

    const addBtn = screen.getByRole('button', { name: /Blacklist author/i });
    await fireEvent.click(addBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/blacklist/author');
    expect(JSON.parse(postCall[1].body as string)).toEqual({ source_id: 2, author_id: 99, reason: 5 });

    await waitFor(() => {
      expect(screen.getByText(/Blacklisted author ffn\/99/)).toBeTruthy();
    });
  });

  it('rejects a non-numeric author id without calling the API', async () => {
    mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, fics: [], authors: [] }) });

    const { default: BlacklistPage } = await import('./+page.svelte');
    render(BlacklistPage);

    const authorInput = await waitFor(() => screen.getByPlaceholderText(/author_id/i));
    await fireEvent.input(authorInput, { target: { value: 'not-a-number' } });

    const addBtn = screen.getByRole('button', { name: /Blacklist author/i });
    await fireEvent.click(addBtn);

    await waitFor(() => {
      expect(screen.getByText(/Author ID must be a positive integer/)).toBeTruthy();
    });
    expect(mockFetch.mock.calls.filter((c) => c[1]?.method === 'POST')).toHaveLength(0);
  });

  it('shows empty states and an error state', async () => {
    mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, fics: [], authors: [] }) });

    const { default: BlacklistPage } = await import('./+page.svelte');
    render(BlacklistPage);

    await waitFor(() => {
      expect(screen.getByText(/No blacklisted fics/)).toBeTruthy();
    });
    expect(screen.getAllByText(/No blacklisted authors/).length).toBeGreaterThan(0);
  });

  it('shows an error message when the API fails', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500 });

    const { default: BlacklistPage } = await import('./+page.svelte');
    render(BlacklistPage);

    await waitFor(() => {
      expect(screen.getAllByText(/Failed to load blacklist/).length).toBeGreaterThan(0);
    });
  });
});
