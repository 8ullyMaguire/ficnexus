import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const mockGoto = vi.fn();
vi.mock('$app/navigation', () => ({
  goto: (path: string) => mockGoto(path),
}));

let mockLoggedIn = true;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockImplementation(async () => {}),
    get isLoggedIn() {
      return mockLoggedIn;
    },
    get user() {
      return mockLoggedIn ? { id: 1, username: 'curator', role: 10, reputation: 0, level: 100 } : null;
    },
    get level() {
      return mockLoggedIn ? 100 : 0;
    },
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

// Import the page lazily so mocks are in place first.
async function loadPage() {
  return import('./+page.svelte');
}

beforeEach(() => {
  mockFetch.mockReset();
  mockGoto.mockReset();
  mockLoggedIn = true;
});

describe('curator coverage page', () => {
  it('renders per-locale progress bars from the coverage matrix', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/translate/coverage')) {
        return Promise.resolve(jsonResponse({
          err: 0,
          coverage: {
            es: { approved: 6, machine: 4 },
            de: { approved: 1, machine: 1 },
          },
          total_targets: 10,
        }));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: Page } = await loadPage();
    render(Page);

    // Locale rows appear with aggregate percentages (6+4 of 10 = 100%, 1+1 of 10 = 20%).
    await waitFor(() => expect(screen.getByText('es')).toBeTruthy());
    expect(screen.getByText('de')).toBeTruthy();
    expect(screen.getByText('100%')).toBeTruthy();
    expect(screen.getByText('20%')).toBeTruthy();
    // Detail line shows approved vs machine split.
    expect(screen.getByText('6 approved · 4 machine')).toBeTruthy();
    expect(screen.getByText('1 approved · 1 machine')).toBeTruthy();
    // Accessible progress bars exist.
    expect(screen.getAllByRole('progressbar').length).toBe(2);
  });

  it('redirects non-curators home', async () => {
    mockLoggedIn = false;
    const { default: Page } = await loadPage();
    render(Page);
    await waitFor(() => expect(mockGoto).toHaveBeenCalledWith('/'));
    // No coverage fetch fired.
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('shows the empty state when nothing is translated', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/translate/coverage')) {
        return Promise.resolve(jsonResponse({ err: 0, coverage: {}, total_targets: 0 }));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: Page } = await loadPage();
    render(Page);
    await waitFor(() => expect(screen.getAllByText(/no translations recorded yet/i).length).toBeGreaterThan(0));
  });
});
