import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    level: 12,
    exp: 4800,
    user: { id: 1, username: 'alice', role: 0, reputation: 0, level: 12, exp: 4800, email: null, locale: 'en' },
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('settings page — F7 level display', () => {
  it('renders the level badge and exp progress bar from /api/users/me/level', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/users/me/level')) {
        return Promise.resolve(
          jsonResponse({ err: 0, level: 12, exp: 4800, exp_to_next: 200, progress: 0.5, level_up: false }),
        );
      }
      if (url.includes('/api/user/site-credentials')) {
        return Promise.resolve(jsonResponse({ err: 0, credentials: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Level 12/)).toBeTruthy();
    });
    expect(screen.getAllByText(/4800/).length).toBeGreaterThan(0);
    // Progress bar at 50% (progress 0.5 → width 50%)
    const fill = document.querySelector('.archive-progress-fill') as HTMLElement | null;
    expect(fill?.style.width).toBe('50%');
    expect(screen.getAllByText(/200/).length).toBeGreaterThan(0);
  });

  it('shows the max-level message instead of a progress bar at level 100', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/users/me/level')) {
        return Promise.resolve(
          jsonResponse({ err: 0, level: 100, exp: 50000, exp_to_next: 0, progress: 1, level_up: false }),
        );
      }
      if (url.includes('/api/user/site-credentials')) {
        return Promise.resolve(jsonResponse({ err: 0, credentials: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Level 100/)).toBeTruthy();
    });
    expect(document.querySelector('.archive-progress-fill')).toBeNull();
  });

  it('surfaces an error when the level endpoint fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/users/me/level')) {
        return Promise.resolve(jsonResponse({ err: -1 }));
      }
      if (url.includes('/api/user/site-credentials')) {
        return Promise.resolve(jsonResponse({ err: 0, credentials: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Could not load your level/)).toBeTruthy();
    });
  });
});

describe('settings page — personalized recommendations toggle', () => {
  function prefsMockImplementation(opts: { prefValue?: string } = {}) {
    return (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes('/api/me/prefs') && (!init || !init.method || init.method === 'GET')) {
        const prefs = [] as { key: string; value: string }[];
        if (opts.prefValue !== undefined) prefs.push({ key: 'recs.personalized', value: opts.prefValue });
        return Promise.resolve(jsonResponse(prefs));
      }
      if (url.includes('/api/users/me/level')) {
        return Promise.resolve(
          jsonResponse({ err: 0, level: 12, exp: 4800, exp_to_next: 200, progress: 0.5, level_up: false }),
        );
      }
      if (url.includes('/api/user/site-credentials')) {
        return Promise.resolve(jsonResponse({ err: 0, credentials: [] }));
      }
      // Notification prefs / blacklist / timezone endpoints — empty ok responses.
      if (url.startsWith('/api/')) {
        return Promise.resolve(jsonResponse({ err: 0 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    };
  }

  it('renders the personalized recommendations toggle', async () => {
    mockFetch.mockImplementation(prefsMockImplementation({ prefValue: 'true' }));
    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Personalized recommendations')).toBeTruthy();
    });
    const checkboxes = screen.getAllByRole('checkbox') as HTMLInputElement[];
    const toggle = checkboxes.find(
      (c) => c.closest('.toggle-row, .archive-checkbox-label') && c.checked,
    );
    expect(toggle?.checked).toBe(true);
  });

  it('reflects an explicit opt-out from /api/me/prefs', async () => {
    mockFetch.mockImplementation(prefsMockImplementation({ prefValue: 'false' }));
    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Personalized recommendations')).toBeTruthy();
    });
    const checkboxes = screen.getAllByRole('checkbox') as HTMLInputElement[];
    const toggles = checkboxes.filter(
      (c) => c.closest('.toggle-row, .archive-checkbox-label') &&
             /reading history/i.test(c.closest('label')?.textContent ?? ''),
    );
    expect(toggles.length).toBeGreaterThan(0);
    await waitFor(() => {
      expect(toggles.every((c) => !c.checked)).toBe(true);
    });
  });

  it('PUTs the new value when the toggle changes', async () => {
    mockFetch.mockImplementation(prefsMockImplementation({ prefValue: 'true' }));
    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Personalized recommendations')).toBeTruthy();
    });
    const checkboxes = screen.getAllByRole('checkbox') as HTMLInputElement[];
    const toggle = checkboxes.find((c) => c.closest('.toggle-row, .archive-checkbox-label') && c.checked);
    expect(toggle).toBeTruthy();

    const putCallsBefore = mockFetch.mock.calls.filter(
      ([, init]) => (init as RequestInit | undefined)?.method === 'PUT',
    ).length;
    // fireEvent.change sets checked AND dispatches change — Svelte's
    // bind:checked reacts to it (a bare dispatchEvent doesn't reach Svelte 5).
    await fireEvent.change(toggle!, { target: { checked: false } });

    await waitFor(() => {
      const puts = mockFetch.mock.calls.filter(
        ([url, init]) =>
          String(url).includes('/api/me/prefs') && (init as RequestInit | undefined)?.method === 'PUT',
      );
      expect(puts.length).toBe(putCallsBefore + 1);
      const body = JSON.parse((puts[puts.length - 1][1] as RequestInit).body as string);
      expect(Array.isArray(body) ? body[0] : body).toEqual({
        key: 'recs.personalized',
        value: 'false',
      });
    });
  });
});
